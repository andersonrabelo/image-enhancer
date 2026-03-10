#!/bin/bash

# Define cores para os prints
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Image Enhancer: Iniciando Setup para Linux ===${NC}"

# Verifica se o Rust está instalado
if ! command -v cargo &> /dev/null; then
    echo "Rust não encontrado. Por favor, instale o Rust usando: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Verifica se o Node.js está instalado
if ! command -v npm &> /dev/null; then
    echo "Node.js (npm) não encontrado. Por favor, instale o Node.js antes de continuar."
    exit 1
fi

echo -e "${GREEN}1. Instalando dependências do Frontend e construindo...${NC}"
npm install
npm run build

echo -e "${GREEN}2. Compilando o Backend (Rust)...${NC}"
cd src-tauri
cargo build --release
cd ..

echo -e "${GREEN}3. Configurando pastas...${NC}"
mkdir -p uploads
mkdir -p src-tauri/models

echo -e "${BLUE}=== Setup Concluído ===${NC}"
echo -e "Iniciando o servidor na porta 8080..."

# Inicia o servidor em background
./src-tauri/target/release/image-enhancer-server &
BACKEND_PID=$!

sleep 2

echo -e "${GREEN}O servidor está rodando na porta 8080!${NC}"
echo ""
echo "Você pode expor para internet usando Cloudflare Tunnel (cloudflared):"
echo "    cloudflared tunnel --url http://localhost:8080"
echo ""
echo "Ou usando localtunnel:"
echo "    npx localtunnel --port 8080"
echo ""
echo "Acesse a URL gerada e aproveite o sistema remotamente!"
echo "(Para desligar o servidor, aperte Ctrl+C)"

# Espera até o usuário apertar Ctrl+C para matar tudo
trap "kill $BACKEND_PID; exit" INT TERM
wait $BACKEND_PID
