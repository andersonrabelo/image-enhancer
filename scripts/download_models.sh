#!/bin/bash

# Define cores para os prints
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

MODELS_DIR="src-tauri/models"

echo -e "${BLUE}=== Image Enhancer: Baixando Modelos de IA (ONNX) ===${NC}"

# Cria diretório de modelos se não existir
mkdir -p "$MODELS_DIR"

# Função utilitária para baixar arquivo se ele não existir
download_model() {
    local url=$1
    local filename=$2
    local filepath="$MODELS_DIR/$filename"

    if [ -f "$filepath" ]; then
        echo -e "${GREEN}✓ Modelo já existe: $filename${NC}"
    else
        echo -e "${BLUE}⬇ Baixando $filename...${NC}"
        # Usa wget ou curl
        if command -v wget &> /dev/null; then
            wget -q --show-progress -O "$filepath" "$url"
        elif command -v curl &> /dev/null; then
            curl -L -o "$filepath" "$url"
        else
            echo -e "${RED}Erro: Nem 'wget' nem 'curl' estão instalados. Por favor, instale um deles para baixar os modelos.${NC}"
            exit 1
        fi
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ Download concluído: $filename${NC}"
        else
            echo -e "${RED}✗ Falha ao baixar: $filename${NC}"
            rm -f "$filepath" # Remove arquivo parcial
        fi
    fi
}

echo "Verificando modelos na pasta $MODELS_DIR..."

# --- Links para Download dos Modelos ONNX ---
URL_YOLOV8="https://huggingface.co/deepghs/yolo-face/resolve/main/yolov8m-face/model.onnx"
URL_CODEFORMER="https://huggingface.co/maze/faceX/resolve/main/codeformer.onnx"
URL_SCUNET="https://huggingface.co/deepghs/image_restoration/resolve/main/SCUNet-PSNR.onnx"
URL_REALESRGAN="https://huggingface.co/AXERA-TECH/Real-ESRGAN/resolve/main/onnx/realesrgan-x4-256.onnx"

download_model "$URL_YOLOV8" "yolov8_face.onnx"
download_model "$URL_CODEFORMER" "codeformer.onnx"
download_model "$URL_SCUNET" "scunet.onnx"
download_model "$URL_REALESRGAN" "realesrgan-x4-256.onnx"

echo -e "${BLUE}=== Download de Modelos Concluído ===${NC}"
