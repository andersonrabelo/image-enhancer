#!/bin/bash

# Define cores para os prints
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Image Enhancer: Iniciando Setup para Linux ===${NC}"

# Verifica se o Rust está instalado
if ! command -v cargo &> /dev/null; then
    echo -e "${BLUE}Rust não encontrado. Instalando automaticamente...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Função para verificar versão do Node.js
check_node_version() {
    if ! command -v node &> /dev/null; then return 1; fi
    local version=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
    if [ "$version" -lt 18 ]; then return 1; fi
    return 0
}

# Verifica se o Node.js está instalado e é moderno (v18+)
if ! check_node_version; then
    echo -e "${BLUE}Node.js não encontrado ou muito antigo ($(node -v 2>/dev/null || echo "N/A")). Instalando Node.js 20...${NC}"
    if command -v apt-get &> /dev/null; then
        # Garante que temos curl
        apt-get update && apt-get install -y curl
        
        # Usa o instalador oficial da NodeSource para Node 20
        if command -v sudo &> /dev/null; then
            curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
            sudo apt-get install -y nodejs
        else
            curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
            apt-get install -y nodejs
        fi
    else
        echo -e "${RED}Erro: Não foi possível instalar Node.js 18+. Por favor, atualize manualmente.${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}1. Instalando dependências do Frontend e construindo...${NC}"
npm install
npm run build

if [ ! -d "dist" ]; then
    echo -e "${RED}Erro crítico: A pasta 'dist' não foi gerada. O Frontend falhou ao compilar.${NC}"
    exit 1
fi

echo -e "${GREEN}2. Compilando o Backend (Rust)...${NC}"
cd src-tauri
cargo build --release
cd ..

echo -e "${GREEN}3. Configurando pastas...${NC}"
mkdir -p uploads
mkdir -p src-tauri/models

echo -e "${GREEN}4. Baixando Modelos ONNX de IA...${NC}"
chmod +x scripts/download_models.sh
./scripts/download_models.sh

echo -e "${BLUE}=== Setup Concluído ===${NC}"

echo -e "${BLUE}Limpando instâncias fantasmas de execuções anteriores...${NC}"
fuser -k 8080/tcp 2>/dev/null || true
pkill -f image-enhancer-server || true
sleep 1

echo -e "Iniciando o servidor na porta 8080..."

# Inicia o servidor em background
./src-tauri/target/release/image-enhancer-server &
BACKEND_PID=$!

sleep 2

echo -e "${GREEN}O servidor está rodando na porta 8080!${NC}"
echo -e "${GREEN}O servidor está rodando na porta 8080 localmente!${NC}"
echo -e "${BLUE}Iniciando a exposição pública da sua API...${NC}"

# Define função para fechar o backend ao sair
cleanup() {
    echo -e "\n${BLUE}Desligando o servidor e túnel...${NC}"
    pkill -f cloudflared || true
    pkill -f localtunnel || true
    kill $BACKEND_PID
    exit
}
trap cleanup INT TERM

echo -e "${GREEN}-> Iniciando Cloudflare Tunnel (cloudflared)...${NC}"
if command -v cloudflared &> /dev/null; then
    CLOUDFLARED_BIN="cloudflared"
else
    echo -e "${BLUE}cloudflared não encontrado. Baixando binário autônomo...${NC}"
    if [ "$(uname -m)" = "aarch64" ]; then ARCH="arm64"; else ARCH="amd64"; fi
    curl -sL --output cloudflared "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-${ARCH}"
    chmod +x cloudflared
    CLOUDFLARED_BIN="./cloudflared"
fi

$CLOUDFLARED_BIN tunnel --url http://127.0.0.1:8080
TUNNEL_EXIT=$?

if [ $TUNNEL_EXIT -ne 0 ]; then
    echo -e "${GREEN}-> Usando Localtunnel como alternativa...${NC}"
    echo -e "Acesse o link gerado abaixo e permita a conexão na tela do LocalTunnel."
    npx localtunnel --port 8080 --local-host 127.0.0.1
fi

cleanup
