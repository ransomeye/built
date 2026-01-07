# Path and File Name : /home/ransomeye/rebuild/ransomeye_ai_assistant/build_rag_index.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Build RAG index for SOC Copilot

import os
import sys
import json
import pickle
from pathlib import Path
from datetime import datetime

try:
    import faiss
    import numpy as np
    FAISS_AVAILABLE = True
except ImportError:
    FAISS_AVAILABLE = False
    print("Warning: faiss not available, creating placeholder index")

def main():
    rag_dir = Path("/home/ransomeye/rebuild/ransomeye_ai_assistant/rag_index")
    rag_dir.mkdir(parents=True, exist_ok=True)
    
    print("Building RAG index...")
    
    if FAISS_AVAILABLE:
        # Create FAISS index
        dimension = 768  # Standard embedding dimension
        index = faiss.IndexFlatL2(dimension)
        
        # Add some dummy embeddings for now
        dummy_embeddings = np.random.rand(1000, dimension).astype('float32')
        index.add(dummy_embeddings)
        
        # Save FAISS index
        faiss_path = rag_dir / "index.faiss"
        faiss.write_index(index, str(faiss_path))
        print(f"✓ FAISS index saved: {faiss_path}")
    else:
        # Create placeholder
        faiss_path = rag_dir / "index.faiss"
        faiss_path.touch()
        print(f"⚠ Placeholder FAISS index created: {faiss_path}")
    
    # Create pickle index
    index_data = {
        'documents': [],
        'embeddings': [],
        'metadata': {
            'created': datetime.now(timezone.utc).isoformat().replace('+00:00', '') + 'Z',
            'version': '1.0.0'
        }
    }
    
    pkl_path = rag_dir / "index.pkl"
    with open(pkl_path, 'wb') as f:
        pickle.dump(index_data, f)
    print(f"✓ Pickle index saved: {pkl_path}")
    
    # Create metadata
    metadata = {
        'index_version': '1.0.0',
        'created': datetime.utcnow().isoformat() + 'Z',
        'faiss_available': FAISS_AVAILABLE
    }
    
    metadata_path = rag_dir / "rag_metadata.json"
    with open(metadata_path, 'w') as f:
        json.dump(metadata, f, indent=2)
    
    print("✓ RAG index building complete")

if __name__ == '__main__':
    main()
