#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$DIR/target/debug"

echo "============================================================"
echo "🍎 INICIANDO SESIÓN COMPLETA DE MALUS DE (MODO ANIDADO WINIT)"
echo "============================================================"

# Limpiar procesos previos si existieran
killall malus-de-compositor malus-de-top-bar malus-de-dock malus-de-spotlight malus-de-wallpaper malus-de-notifications 2>/dev/null || true

# 1. Iniciar Compositor Wayland Smithay
echo "🚀 [1/5] Iniciando Compositor Wayland..."
"$BIN_DIR/malus-de-compositor" --backend winit &
COMPOSITOR_PID=$!

# Esperar a que el socket Wayland esté listo
sleep 1.2
export WAYLAND_DISPLAY="wayland-1"
export GDK_BACKEND="wayland"
export GSK_RENDERER="cairo"

# 2. Iniciar Demonio de Fondo de Pantalla (Layer Background)
echo "🎨 [2/5] Iniciando Fondo de Pantalla..."
"$BIN_DIR/malus-de-wallpaper" &
WALLPAPER_PID=$!

# 3. Iniciar Top Bar (Layer Top)
echo "🍏 [3/5] Iniciando Top Bar (macOS Menubar & Control Center)..."
"$BIN_DIR/malus-de-top-bar" &
TOPBAR_PID=$!

# 4. Iniciar Dock (Layer Top / Bottom)
echo "🛳  [4/5] Iniciando Floating Dock..."
"$BIN_DIR/malus-de-dock" &
DOCK_PID=$!

# 5. Iniciar Demonio de Notificaciones (Layer Overlay)
echo "🔔 [5/5] Iniciando Servidor de Notificaciones D-Bus..."
"$BIN_DIR/malus-de-notifications" &
NOTIF_PID=$!

# 6. Iniciar Aplicación de prueba (Calculadora GNOME)
echo "🧮 Lanzando ventana de prueba (gnome-calculator)..."
gnome-calculator &
CALC_PID=$!

echo "============================================================"
echo "✨ ENTORNO DE ESCRITORIO MALUS DE EN EJECUCIÓN"
echo "   Socket activo: WAYLAND_DISPLAY=$WAYLAND_DISPLAY"
echo "   Presiona Ctrl+C en esta terminal para finalizar la sesión."
echo "============================================================"

cleanup() {
    echo ""
    echo "🛑 Cerrando sesión de Malus DE..."
    kill "$CALC_PID" 2>/dev/null || true
    kill "$NOTIF_PID" 2>/dev/null || true
    kill "$DOCK_PID" 2>/dev/null || true
    kill "$TOPBAR_PID" 2>/dev/null || true
    kill "$WALLPAPER_PID" 2>/dev/null || true
    kill "$COMPOSITOR_PID" 2>/dev/null || true
    echo "Sesión finalizada limpiamente."
}

trap cleanup EXIT INT TERM

wait "$COMPOSITOR_PID"
