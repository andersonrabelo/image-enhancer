import { useState, useRef } from "react";
import { UploadCloud, Loader2, ArrowLeft, Download, CheckCircle2 } from "lucide-react";
import "./App.css";

interface ProcessResult {
  success: boolean;
  original_path: string;
  result_path: string;
  message: string;
}

function App() {
  const [isHovering, setIsHovering] = useState(false);
  const [selectedImage, setSelectedImage] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const [isProcessing, setIsProcessing] = useState(false);
  const [resultImage, setResultImage] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string>("");

  const handleFileSelect = () => {
    fileInputRef.current?.click();
  };

  const onFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      const file = e.target.files[0];
      setSelectedFile(file);
      setSelectedImage(URL.createObjectURL(file));
      setResultImage(null);
      setSuccessMsg("");
    }
  };

  const handleDragOver = (e: React.DragEvent) => { e.preventDefault(); setIsHovering(true); };
  const handleDragLeave = () => setIsHovering(false);

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsHovering(false);
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      const file = e.dataTransfer.files[0];
      setSelectedFile(file);
      setSelectedImage(URL.createObjectURL(file));
      setResultImage(null);
      setSuccessMsg("");
    }
  };

  const handleProcess = async () => {
    if (!selectedFile) return;
    setIsProcessing(true);
    setSuccessMsg("");

    try {
      const formData = new FormData();
      formData.append("image", selectedFile);
      
      const API_URL = import.meta.env.VITE_API_URL || "";
      console.log("Enviando requisição via HTTP para: ", API_URL || "caminho relativo (mesmo servidor)");
      
      const response = await fetch(`${API_URL}/api/process`, {
        method: "POST",
        body: formData,
      });

      if (!response.ok) throw new Error("Network response was not ok");
      const result: ProcessResult = await response.json();
      console.log("Servidor retornou:", result);

      if (result.success && result.result_path) {
        setSuccessMsg(result.message);
        setResultImage(`${API_URL}/${result.result_path}`);
      }
    } catch (err: any) {
      console.error("Erro no processamento:", err);
      alert("Falha: " + err.message);
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="app-container">
      <header className="header" style={{ marginBottom: resultImage ? '1.5rem' : '3rem' }}>
        <h1>Image Enhancer</h1>
        <p>Restauração avançada de fotos com IA e Processamento Paralelo</p>
      </header>

      <main className="upload-glass-panel" style={{ padding: resultImage ? '2rem' : '3rem' }}>

        {/* State 1: No Image */}
        {!selectedImage && !resultImage && (
          <div
            className={`dropzone ${isHovering ? 'active' : ''}`}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            onClick={handleFileSelect}
          >
            <UploadCloud className="upload-icon" strokeWidth={1.5} />
            <div className="dropzone-text">
              <h3>Solte sua imagem aqui</h3>
              <p>ou clique para procurar em seus arquivos</p>
            </div>
            <button className="action-button">Procurar Arquivo</button>
            <input 
              type="file" 
              ref={fileInputRef} 
              style={{ display: 'none' }} 
              accept="image/png, image/jpeg, image/jpg, image/webp"
              onChange={onFileChange} 
            />
          </div>
        )}

        {/* State 2: Image Selected, Waiting for Process */}
        {selectedImage && !resultImage && (
          <div className="preview-container" style={{ textAlign: 'center', width: '100%' }}>
            <div style={{ position: 'relative', display: 'inline-block' }}>
              <img src={selectedImage} alt="Selected layout" style={{ maxWidth: '100%', maxHeight: '420px', borderRadius: '16px', marginBottom: '20px', boxShadow: '0 10px 30px rgba(0,0,0,0.5)' }} />
              {isProcessing && (
                <div style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: 'calc(100% - 20px)', background: 'rgba(10, 10, 12, 0.7)', borderRadius: '16px', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', backdropFilter: 'blur(8px)' }}>
                  <Loader2 className="processing-spinner" style={{ width: 48, height: 48, color: '#6366f1', animation: 'spin 2s linear infinite' }} />
                  <p style={{ marginTop: '1rem', fontWeight: 500 }}>A Esteira Paralela está trabalhando...</p>
                  <p style={{ color: '#a1a1aa', fontSize: '0.9rem', marginTop: '0.2rem' }}>Acionando Sidecars, alocando GPU e criando Seamless Blends.</p>
                </div>
              )}
            </div>
            <br />

            <button className="action-button" onClick={() => setSelectedImage(null)} disabled={isProcessing} style={{ opacity: isProcessing ? 0.5 : 1 }}>
              <ArrowLeft size={18} style={{ display: 'inline', marginRight: '8px', verticalAlign: 'middle' }} /> Trocar
            </button>
            <button
              className="action-button"
              onClick={handleProcess}
              disabled={isProcessing}
              style={{ marginLeft: '12px', background: isProcessing ? '#3f3f46' : '#10b981', boxShadow: isProcessing ? 'none' : '0 4px 14px 0 rgba(16, 185, 129, 0.39)', opacity: isProcessing ? 0.8 : 1 }}
            >
              {isProcessing ? 'Restaurando...' : 'Restaurar Imagem na GPU'}
            </button>
          </div>
        )}

        {/* State 3: Result Done */}
        {resultImage && (
          <div className="result-container" style={{ width: '100%', animation: 'fadeIn 0.8s ease-out' }}>
            <div style={{ display: 'flex', gap: '20px', justifyContent: 'center', alignItems: 'center', flexWrap: 'wrap' }}>
              <div style={{ flex: '1 1 300px', maxWidth: '400px', textAlign: 'center' }}>
                <p style={{ marginBottom: '8px', color: '#a1a1aa', fontWeight: 500 }}>Original (Baixa Resolução)</p>
                <img src={selectedImage!} alt="Original" style={{ width: '100%', borderRadius: '12px', opacity: 0.6 }} />
              </div>

              <div style={{ flex: '1 1 400px', maxWidth: '550px', textAlign: 'center', transform: 'scale(1.05)' }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '8px', marginBottom: '8px', color: '#10b981', fontWeight: 600 }}>
                  <CheckCircle2 size={18} /> Ultra Alta Definição (Real-ESRGAN)
                </div>
                <img src={resultImage} alt="Enhanced Result" style={{ width: '100%', borderRadius: '16px', boxShadow: '0 20px 40px rgba(16, 185, 129, 0.2), 0 0 0 2px rgba(16, 185, 129, 0.5)' }} />
              </div>
            </div>

            <div style={{ textAlign: 'center', marginTop: '30px' }}>
              <p style={{ color: '#a1a1aa', marginBottom: '20px', fontSize: '0.95rem' }}>{successMsg}</p>
              <button className="action-button" onClick={() => { setResultImage(null); setSelectedImage(null); setSelectedFile(null); }} style={{ background: 'transparent', border: '1px solid var(--border-color)', color: 'white' }}>
                Nova Imagem
              </button>
              <a href={resultImage} download={`enhanced_${selectedFile?.name || 'image'}.jpg`} style={{ textDecoration: 'none' }}>
                <button className="action-button" style={{ marginLeft: '12px', background: '#6366f1' }}>
                  <Download size={18} style={{ display: 'inline', marginRight: '8px', verticalAlign: 'middle' }} />
                  Baixar Imagem!
                </button>
              </a>
            </div>
          </div>
        )}
      </main>

      <style>
        {`@keyframes spin { 100% { transform: rotate(360deg); } }`}
      </style>
    </div>
  );
}

export default App;
