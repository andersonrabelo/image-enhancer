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

# Função utilitária para baixar arquivo
download_model() {
    local url=$1
    local filename=$2
    local filepath="$MODELS_DIR/$filename"

    # Usa wget ou curl
    if command -v wget &> /dev/null; then
        wget -q --show-progress -O "$filepath" "$url"
    elif command -v curl &> /dev/null; then
        curl -L -o "$filepath" "$url"
    else
        echo -e "${RED}Erro: Nem 'wget' nem 'curl' estão instalados.${NC}"
        return 1
    fi
    
    if [ $? -eq 0 ] && [ -f "$filepath" ]; then
        echo -e "${GREEN}✓ Download concluído: $filename${NC}"
        return 0
    else
        echo -e "${RED}✗ Falha ao baixar de: $url${NC}"
        rm -f "$filepath"
        return 1
    fi
}

echo "Verificando modelos na pasta $MODELS_DIR..."

# --- Links para Download dos Modelos ONNX ---
URL_YOLOV8="https://huggingface.co/deepghs/yolo-face/resolve/main/yolov8m-face/model.onnx"
# Primary and Fallback URLs for FULL 377MB CodeFormer (Stable on CPU)
URL_CODEFORMER_1="https://huggingface.co/MonsterMMORPG/SECourses/resolve/main/codeformer.onnx"
URL_CODEFORMER_2="https://huggingface.co/facefusion/models-3.0.0/resolve/main/codeformer.onnx"
URL_SCUNET="https://huggingface.co/deepghs/image_restoration/resolve/main/SCUNet-PSNR.onnx"
URL_REALESRGAN="https://huggingface.co/AXERA-TECH/Real-ESRGAN/resolve/main/onnx/realesrgan-x4-256.onnx"

# Improved download function with fallback
download_with_fallback() {
    local filename=$1
    local url1=$2
    local url2=$3
    local filepath="$MODELS_DIR/$filename"

    # Se o arquivo já existe e é > 300MB (para CodeFormer) ou > 10MB (outros), pulamos
    local min_size=10000000
    if [[ "$filename" == "codeformer.onnx" ]]; then
        min_size=300000000
    fi

    if [ -f "$filepath" ]; then
        local size=$(stat -c%s "$filepath")
        if [ "$size" -ge "$min_size" ]; then
            echo -e "${GREEN}✓ Modelo já existe e parece íntegro: $filename ($((size/1024/1024)) MB)${NC}"
            return 0
        else
            echo -e "${RED}! Modelo $filename incompleto ou versão antiga ($((size/1024/1024)) MB). Redownload...${NC}"
            rm -f "$filepath"
        fi
    fi

    echo -e "${BLUE}⬇ Baixando $filename...${NC}"
    if ! download_model "$url1" "$filename"; then
        if [ -n "$url2" ]; then
            echo -e "${BLUE}⬇ Tentando link alternativo para $filename...${NC}"
            download_model "$url2" "$filename"
        fi
    fi
}

download_with_fallback "yolov8_face.onnx" "$URL_YOLOV8"
download_with_fallback "codeformer.onnx" "$URL_CODEFORMER_1" "$URL_CODEFORMER_2"
download_with_fallback "scunet.onnx" "$URL_SCUNET"
download_with_fallback "realesrgan-x4-256.onnx" "$URL_REALESRGAN"

echo -e "${BLUE}=== Download de Modelos Concluído ===${NC}"
