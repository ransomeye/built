# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/llm_knowledge/build_rag_index.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Builds pre-indexed RAG knowledge base from documents

"""
RAG Index Builder: Builds pre-indexed RAG knowledge base.
Index is built at release time, not at runtime.
"""

import sys
import json
import hashlib
from pathlib import Path
from datetime import datetime
from typing import List, Dict

try:
    import numpy as np
    from sentence_transformers import SentenceTransformer
    import faiss
except ImportError:
    print("ERROR: Required libraries not installed.", file=sys.stderr)
    print("Install with: pip install sentence-transformers faiss-cpu numpy", file=sys.stderr)
    sys.exit(1)

LLM_KNOWLEDGE_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/llm_knowledge")
DOCUMENTS_DIR = LLM_KNOWLEDGE_DIR / "documents"
RAG_INDEX_DIR = LLM_KNOWLEDGE_DIR / "rag_index"


def chunk_text(text: str, chunk_size: int = 512, overlap: int = 50) -> List[str]:
    """
    Split text into overlapping chunks.
    
    Args:
        text: Text to chunk
        chunk_size: Size of each chunk
        overlap: Overlap between chunks
    
    Returns:
        List of text chunks
    """
    chunks = []
    words = text.split()
    
    i = 0
    while i < len(words):
        chunk_words = words[i:i + chunk_size]
        chunk = ' '.join(chunk_words)
        chunks.append(chunk)
        i += chunk_size - overlap
    
    return chunks


def load_documents() -> List[Dict]:
    """Load all documents from documents directory."""
    documents = []
    
    for doc_path in DOCUMENTS_DIR.glob("*.md"):
        with open(doc_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Chunk document
        chunks = chunk_text(content)
        
        documents.append({
            'path': str(doc_path.relative_to(LLM_KNOWLEDGE_DIR)),
            'content': content,
            'chunks': chunks,
            'hash': hashlib.sha256(content.encode()).hexdigest()
        })
    
    return documents


def build_index(documents: List[Dict]) -> faiss.Index:
    """
    Build FAISS index from documents.
    
    Returns:
        FAISS index
    """
    print("Loading embedding model...")
    # Use lightweight model for offline operation
    model = SentenceTransformer('all-MiniLM-L6-v2')
    
    print("Generating embeddings...")
    all_chunks = []
    all_embeddings = []
    
    for doc in documents:
        for chunk in doc['chunks']:
            all_chunks.append({
                'document': doc['path'],
                'chunk': chunk
            })
    
    # Generate embeddings in batches
    batch_size = 32
    for i in range(0, len(all_chunks), batch_size):
        batch_chunks = [c['chunk'] for c in all_chunks[i:i + batch_size]]
        embeddings = model.encode(batch_chunks, show_progress_bar=False)
        all_embeddings.append(embeddings)
        print(f"  Processed {min(i + batch_size, len(all_chunks))}/{len(all_chunks)} chunks")
    
    # Concatenate embeddings
    embeddings_array = np.vstack(all_embeddings).astype('float32')
    
    print(f"Building FAISS index...")
    print(f"  Embeddings shape: {embeddings_array.shape}")
    
    # Create FAISS index
    dimension = embeddings_array.shape[1]
    index = faiss.IndexFlatL2(dimension)
    index.add(embeddings_array)
    
    print(f"  ✓ Index built with {index.ntotal} vectors")
    
    return index, all_chunks


def save_index(index: faiss.Index, chunks: List[Dict], documents: List[Dict]) -> None:
    """Save FAISS index and metadata."""
    RAG_INDEX_DIR.mkdir(parents=True, exist_ok=True)
    
    # Save index
    index_path = RAG_INDEX_DIR / "index.bin"
    faiss.write_index(index, str(index_path))
    print(f"  ✓ Index saved: {index_path}")
    
    # Save chunks metadata
    chunks_path = RAG_INDEX_DIR / "chunks.json"
    with open(chunks_path, 'w') as f:
        json.dump(chunks, f, indent=2)
    print(f"  ✓ Chunks metadata saved: {chunks_path}")
    
    # Update index manifest
    manifest_path = RAG_INDEX_DIR / "index_manifest.json"
    
    # Compute index hash
    index_hash = hashlib.sha256()
    with open(index_path, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b''):
            index_hash.update(chunk)
    index_hash_hex = index_hash.hexdigest()
    
    manifest = {
        'index_version': '1.0.0',
        'index_type': 'faiss',
        'index_file': 'index.bin',
        'created': datetime.utcnow().isoformat() + 'Z',
        'document_count': len(documents),
        'chunk_count': len(chunks),
        'embedding_dimension': index.d,
        'embedding_model': 'sentence-transformers/all-MiniLM-L6-v2',
        'index_hash': f'sha256:{index_hash_hex}',
        'documents': [
            {
                'path': doc['path'],
                'chunks': len(doc['chunks']),
                'hash': doc['hash']
            }
            for doc in documents
        ],
        'signature': {
            'algorithm': 'RSA-4096-PSS-SHA256',
            'signer': 'ransomeye_rag_indexer',
            'signature': 'placeholder',  # Will be updated after signing
            'timestamp': datetime.utcnow().isoformat() + 'Z'
        }
    }
    
    with open(manifest_path, 'w') as f:
        json.dump(manifest, f, indent=2)
    
    print(f"  ✓ Index manifest updated: {manifest_path}")


def main():
    """Build RAG index from documents."""
    print("=" * 80)
    print("RansomEye Intelligence - RAG Index Builder")
    print("=" * 80)
    print()
    print("Building pre-indexed RAG knowledge base...")
    print("Index is built at release time, not at runtime.")
    print()
    
    # Load documents
    print("Loading documents...")
    documents = load_documents()
    print(f"  ✓ Loaded {len(documents)} documents")
    for doc in documents:
        print(f"    - {doc['path']}: {len(doc['chunks'])} chunks")
    print()
    
    # Build index
    index, chunks = build_index(documents)
    print()
    
    # Save index
    save_index(index, chunks, documents)
    print()
    
    print("=" * 80)
    print("✓ RAG index build complete")
    print("=" * 80)


if __name__ == '__main__':
    main()

