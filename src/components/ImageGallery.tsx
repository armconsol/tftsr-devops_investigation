import React, { useState, useRef, useEffect } from "react";
import { X, AlertTriangle, ExternalLink, Image as ImageIcon } from "lucide-react";
import type { ImageAttachment } from "@/lib/tauriCommands";

interface ImageGalleryProps {
  images: ImageAttachment[];
  onDelete?: (attachment: ImageAttachment) => void;
  showWarning?: boolean;
}

export function ImageGallery({ images, onDelete, showWarning = true }: ImageGalleryProps) {
  const [selectedImage, setSelectedImage] = useState<ImageAttachment | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const modalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isModalOpen) {
        setIsModalOpen(false);
        setSelectedImage(null);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isModalOpen]);

  if (images.length === 0) return null;

  const base64ToDataUrl = (base64: string, mimeType: string): string => {
    if (base64.startsWith("data:image/")) {
      return base64;
    }
    return `data:${mimeType};base64,${base64}`;
  };

  const getPreviewUrl = (attachment: ImageAttachment): string => {
    if (attachment.file_path && attachment.file_path.length > 0) {
      return `file://${attachment.file_path}`;
    }
    return base64ToDataUrl(attachment.upload_hash, attachment.mime_type);
  };

  const isWebSource = (image: ImageAttachment): boolean => {
    return image.file_path.length > 0 && 
           (image.file_path.startsWith("http://") || 
            image.file_path.startsWith("https://"));
  };

  return (
    <div className="space-y-4">
      {showWarning && (
        <div className="bg-amber-100 border border-amber-300 text-amber-800 p-3 rounded-md flex items-center gap-2">
          <AlertTriangle className="w-5 h-5 flex-shrink-0" />
          <span className="text-sm">
            ⚠️ PII cannot be automatically redacted from images. Use at your own risk.
          </span>
        </div>
      )}

      {images.some(img => isWebSource(img)) && (
        <div className="bg-red-100 border border-red-300 text-red-800 p-3 rounded-md flex items-center gap-2">
          <ExternalLink className="w-5 h-5 flex-shrink-0" />
          <span className="text-sm">
            ⚠️ Some images appear to be from web sources. Ensure you have permission to share.
          </span>
        </div>
      )}

      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
        {images.map((image) => (
          <div key={image.id} className="group relative rounded-lg overflow-hidden bg-gray-100 border border-gray-200">
            <button
              onClick={() => {
                setSelectedImage(image);
                setIsModalOpen(true);
              }}
              className="w-full aspect-video object-cover"
            >
              <img
                src={getPreviewUrl(image)}
                alt={image.file_name}
                className="w-full h-full object-cover transition-transform group-hover:scale-110"
                loading="lazy"
              />
            </button>
            <div className="p-2">
              <p className="text-xs text-gray-700 truncate" title={image.file_name}>
                {image.file_name}
              </p>
              <p className="text-xs text-gray-500">
                {image.is_paste ? "Paste" : "Upload"} · {(image.file_size / 1024).toFixed(1)} KB
              </p>
            </div>
            {onDelete && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(image);
                }}
                className="absolute top-1 right-1 p-1 bg-white/80 hover:bg-white rounded-md text-gray-600 hover:text-red-600 transition-colors opacity-0 group-hover:opacity-100"
                title="Delete image"
              >
                <X className="w-4 h-4" />
              </button>
            )}
          </div>
        ))}
      </div>

      {isModalOpen && selectedImage && (
        <div
          className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
          onClick={() => {
            setIsModalOpen(false);
            setSelectedImage(null);
          }}
        >
          <div
            ref={modalRef}
            className="bg-white rounded-lg overflow-hidden max-w-4xl max-h-[90vh] flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="bg-gray-100 p-4 flex items-center justify-between border-b">
              <div className="flex items-center gap-2">
                <ImageIcon className="w-5 h-5 text-gray-600" />
                <h3 className="font-medium">{selectedImage.file_name}</h3>
              </div>
              <button
                onClick={() => {
                  setIsModalOpen(false);
                  setSelectedImage(null);
                }}
                className="p-2 hover:bg-gray-200 rounded-lg transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="flex-1 overflow-auto bg-gray-900 flex items-center justify-center p-8">
              <img
                src={getPreviewUrl(selectedImage)}
                alt={selectedImage.file_name}
                className="max-w-full max-h-[60vh] object-contain"
              />
            </div>
            <div className="bg-gray-50 p-4 border-t text-sm space-y-2">
              <div className="flex gap-4">
                <div>
                  <span className="text-gray-500">Type:</span> {selectedImage.mime_type}
                </div>
                <div>
                  <span className="text-gray-500">Size:</span> {(selectedImage.file_size / 1024).toFixed(2)} KB
                </div>
                <div>
                  <span className="text-gray-500">Source:</span> {selectedImage.is_paste ? "Paste" : "File"}
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default ImageGallery;
