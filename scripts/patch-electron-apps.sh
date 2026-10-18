#!/usr/bin/env bash
# ==============================================================================
# Malus DE - Integración automática de controles de ventana macOS para Electron
# (Visual Studio Code, Antigravity IDE, VSCodium, etc.)
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Detectar usuario objetivo no-root (para rutas de usuario y configuraciones)
TARGET_USER="${SUDO_USER:-$USER}"
TARGET_HOME=$(getent passwd "$TARGET_USER" | cut -d: -f6)

echo "============================================================"
echo "🍎 MALUS DE: PARCHEADOR DE CONTROLES DE VENTANA (ELECTRON)"
echo "   Usuario: $TARGET_USER (Home: $TARGET_HOME)"
echo "============================================================"

THEME_CSS="$PROJECT_DIR/assets/themes/electron-window-controls.css"
if [ ! -f "$THEME_CSS" ]; then
    echo "❌ Error: No se encontró la hoja de estilos $THEME_CSS"
    exit 1
fi

# Asegurar copia en la carpeta de configuración de temas de Malus DE
USER_THEME_DIR="$TARGET_HOME/.config/malus-de/themes"
mkdir -p "$USER_THEME_DIR"
cp -f "$THEME_CSS" "$USER_THEME_DIR/electron-window-controls.css"
if [ "$(id -u)" -eq 0 ]; then
    chown -R "$TARGET_USER:$TARGET_USER" "$TARGET_HOME/.config/malus-de"
fi

# Lista de directorios 'resources/app' candidatos a escanear
CANDIDATES=(
    "$TARGET_HOME/.local/share/antigravity-ide/resources/app:Antigravity IDE (Local)"
    "/usr/share/code/resources/app:Visual Studio Code (Sistema)"
    "/usr/lib/code/resources/app:Visual Studio Code (Lib)"
    "/opt/visual-studio-code/resources/app:Visual Studio Code (Opt)"
    "$TARGET_HOME/.local/share/code/resources/app:Visual Studio Code (Local)"
    "/usr/share/vscodium/resources/app:VSCodium (Sistema)"
    "/usr/share/codium/resources/app:VSCodium (Opt)"
    "$TARGET_HOME/.local/share/vscodium/resources/app:VSCodium (Local)"
)

# Función para actualizar settings.json asegurando titleBarStyle y controlsStyle custom
update_settings_json() {
    local settings_file="$1"
    local dir
    dir="$(dirname "$settings_file")"
    mkdir -p "$dir"
    
    python3 -c "
import json, os

path = '''$settings_file'''
data = {}
if os.path.exists(path):
    try:
        with open(path, 'r', encoding='utf-8') as f:
            data = json.load(f)
    except Exception:
        data = {}

data['window.titleBarStyle'] = 'custom'
data['window.controlsStyle'] = 'custom'

with open(path, 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=4)
"
    if [ "$(id -u)" -eq 0 ]; then
        chown "$TARGET_USER:$TARGET_USER" "$settings_file"
    fi
    echo "      ⚙️ Configuración actualizada: $settings_file"
}

# Función para aplicar el parche en una app Electron de tipo VS Code / Monaco
patch_app() {
    local app_dir="$1"
    local app_name="$2"
    local js_file="$app_dir/out/vs/workbench/workbench.desktop.main.js"
    local css_file="$app_dir/out/vs/workbench/workbench.desktop.main.css"

    if [ ! -f "$js_file" ] || [ ! -f "$css_file" ]; then
        return 0
    fi

    echo ""
    echo "🔍 Aplicación detectada: $app_name"
    echo "   Ruta: $app_dir"

    # Verificar permisos de escritura
    if [ ! -w "$app_dir/out/vs/workbench" ] || [ ! -w "$js_file" ] || [ ! -w "$css_file" ]; then
        if [ "$(id -u)" -ne 0 ]; then
            echo "   ⚠️ Esta ruta requiere permisos de administrador (root)."
            echo "   💡 Ejecuta: sudo bash '$0' para parchear esta aplicación."
            return 0
        fi
    fi

    # 1. Crear respaldos .bak si no existen
    if [ ! -f "$js_file.bak" ]; then
        cp -p "$js_file" "$js_file.bak"
        echo "   💾 Respaldo creado: workbench.desktop.main.js.bak"
    fi
    if [ ! -f "$css_file.bak" ]; then
        cp -p "$css_file" "$css_file.bak"
        echo "   💾 Respaldo creado: workbench.desktop.main.css.bak"
    fi

    # 2. Parchear JavaScript (Mover controles a la izquierda manteniendo longitud exacta)
    python3 -c "
import re

js_path = '''$js_file'''
with open(js_path, 'r', encoding='utf-8') as f:
    content = f.read()

# Patrón: let <var>=<cond>?\"left\":\"right\";
pattern = re.compile(r'let ([a-zA-Z0-9_$]+)=([a-zA-Z0-9_$]+)\?\"left\":\"right\";')
match = pattern.search(content)

if match:
    var_name = match.group(1)
    original = match.group(0)
    # Rellenar con espacios para que la longitud exacta sea de 24 caracteres
    replacement = f'let {var_name}=\"left\"'.ljust(len(original) - 1) + ';'
    assert len(original) == len(replacement), f'Length mismatch: {len(original)} vs {len(replacement)}'
    new_content = content[:match.start()] + replacement + content[match.end():]
    with open(js_path, 'w', encoding='utf-8') as f:
        f.write(new_content)
    print('   ✔ JavaScript parcheado exitosamente (controles reubicados a la izquierda).')
else:
    # Comprobar si ya estaba parcheado
    if re.search(r'let [a-zA-Z0-9_$]+=\"left\"\s*;', content):
        print('   ℹ JavaScript ya se encuentra parcheado a la izquierda.')
    else:
        print('   ⚠️ No se encontró el patrón de controles en el bundle JS.')
"

    # 3. Parchear CSS (Estilo macOS + corrección botón de ayuda en Antigravity)
    python3 -c "
css_path = '''$css_file'''
theme_path = '''$THEME_CSS'''

with open(theme_path, 'r', encoding='utf-8') as f:
    theme_css = f.read().strip()

with open(css_path, 'r', encoding='utf-8') as f:
    css_content = f.read()

marker = '/* =========================================================================='
if marker in css_content:
    clean_css = css_content[:css_content.find(marker)].rstrip()
else:
    clean_css = css_content.rstrip()

final_css = clean_css + '\n\n' + theme_css + '\n'

with open(css_path, 'w', encoding='utf-8') as f:
    f.write(final_css)

print('   ✔ CSS actualizado con el tema de semáforo macOS y posición de ayuda corregida.')
"
}

# Procesar cada candidato detectado
FOUND_COUNT=0
for entry in "${CANDIDATES[@]}"; do
    path="${entry%%:*}"
    name="${entry##*:}"
    if [ -d "$path" ]; then
        patch_app "$path" "$name"
        FOUND_COUNT=$((FOUND_COUNT + 1))
    fi
done

# Actualizar configuraciones de usuario para cada aplicación conocida
echo ""
echo "📝 Sincronizando configuraciones de ventana en settings.json..."
SETTINGS_LIST=(
    "$TARGET_HOME/.config/Code/User/settings.json"
    "$TARGET_HOME/.config/malus-de/Code/User/settings.json"
    "$TARGET_HOME/.config/Antigravity IDE/User/settings.json"
    "$TARGET_HOME/.config/malus-de/Antigravity IDE/User/settings.json"
    "$TARGET_HOME/.config/VSCodium/User/settings.json"
    "$TARGET_HOME/.config/malus-de/VSCodium/User/settings.json"
)

for sf in "${SETTINGS_LIST[@]}"; do
    update_settings_json "$sf"
done

echo ""
echo "============================================================"
echo "✨ PROCESO COMPLETADO ($FOUND_COUNT aplicaciones procesadas)"
echo "   Para aplicar los cambios, reinicia las aplicaciones abiertas."
echo "============================================================"
