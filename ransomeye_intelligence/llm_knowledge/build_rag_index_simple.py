# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/llm_knowledge/build_rag_index_simple.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Builds pre-indexed RAG knowledge base using lightweight approach

"""
RAG Index Builder (Lightweight): Builds pre-indexed RAG knowledge base.
Uses TF-IDF instead of neural embeddings to avoid large dependencies.
"""

import sys
import json
import hashlib
import pickle
from pathlib import Path
from datetime import datetime
from typing import List, Dict
from collections import Counter
import math

try:
    import numpy as np
except ImportError:
    print("ERROR: numpy not installed. Install with: pip install numpy", file=sys.stderr)
    sys.exit(1)

LLM_KNOWLEDGE_DIR = Path("/home/ransomeye/rebuild/ransomeye_intelligence/llm_knowledge")
DOCUMENTS_DIR = LLM_KNOWLEDGE_DIR / "documents"
RAG_INDEX_DIR = LLM_KNOWLEDGE_DIR / "rag_index"


def chunk_text(text: str, chunk_size: int = 512, overlap: int = 50) -> List[str]:
    """Split text into overlapping chunks."""
    chunks = []
    words = text.split()
    
    i = 0
    while i < len(words):
        chunk_words = words[i:i + chunk_size]
        chunk = ' '.join(chunk_words)
        chunks.append(chunk)
        i += chunk_size - overlap
    
    return chunks


def compute_tfidf_embeddings(documents: List[Dict]) -> tuple:
    """
    Compute TF-IDF embeddings for documents.
    
    Returns:
        Tuple of (embeddings_array, vocabulary, idf_scores)
    """
    # Build vocabulary
    all_words = []
    for doc in documents:
        for chunk in doc['chunks']:
            words = chunk.lower().split()
            all_words.extend(words)
    
    vocabulary = list(set(all_words))
    vocab_size = len(vocabulary)
    vocab_index = {word: idx for idx, word in enumerate(vocabulary)}
    
    # Compute IDF
    doc_count = sum(len(doc['chunks']) for doc in documents)
    word_doc_counts = Counter()
    for doc in documents:
        doc_words = set()
        for chunk in doc['chunks']:
            words = chunk.lower().split()
            doc_words.update(words)
        for word in doc_words:
            word_doc_counts[word] += 1
    
    idf_scores = {}
    for word in vocabulary:
        df = word_doc_counts.get(word, 1)
        idf_scores[word] = math.log(doc_count / (df + 1))
    
    # Compute TF-IDF for each chunk
    all_chunks = []
    embeddings = []
    
    for doc in documents:
        for chunk in doc['chunks']:
            all_chunks.append({
                'document': doc['path'],
                'chunk': chunk
            })
            
            # Compute TF
            words = chunk.lower().split()
            word_counts = Counter(words)
            chunk_length = len(words)
            
            # Build TF-IDF vector
            tfidf_vector = np.zeros(vocab_size)
            for word, count in word_counts.items():
                if word in vocab_index:
                    tf = count / chunk_length
                    idf = idf_scores[word]
                    tfidf_vector[vocab_index[word]] = tf * idf
            
            embeddings.append(tfidf_vector)
    
    embeddings_array = np.array(embeddings, dtype='float32')
    
    return embeddings_array, all_chunks, vocabulary, idf_scores


def save_index_simple(embeddings: np.ndarray, chunks: List[Dict], documents: List[Dict], vocabulary: List[str], idf_scores: Dict) -> None:
    """Save TF-IDF index and metadata."""
    RAG_INDEX_DIR.mkdir(parents=True, exist_ok=True)
    
    # Save index as numpy array (simpler than FAISS for this use case)
    index_path = RAG_INDEX_DIR / "index.bin"
    with open(index_path, 'wb') as f:
        pickle.dump(embeddings, f)
    print(f"  ✓ Index saved: {index_path}")
    
    # Save vocabulary and IDF scores
    vocab_path = RAG_INDEX_DIR / "vocabulary.pkl"
    with open(vocab_path, 'wb') as f:
        pickle.dump({'vocabulary': vocabulary, 'idf_scores': idf_scores}, f)
    print(f"  ✓ Vocabulary saved: {vocab_path}")
    
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
        for chunk_data in iter(lambda: f.read(4096), b''):
            index_hash.update(chunk_data)
    index_hash_hex = index_hash.hexdigest()
    
    manifest = {
        'index_version': '1.0.0',
        'index_type': 'tfidf',
        'index_file': 'index.bin',
        'created': datetime.utcnow().isoformat() + 'Z',
        'document_count': len(documents),
        'chunk_count': len(chunks),
        'embedding_dimension': embeddings.shape[1],
        'embedding_model': 'tfidf',
        'vocabulary_size': len(vocabulary),
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
            'signature': 'placeholder',
            'timestamp': datetime.utcnow().isoformat() + 'Z'
        }
    }
    
    with open(manifest_path, 'w') as f:
        json.dump(manifest, f, indent=2)
    
    print(f"  ✓ Index manifest updated: {manifest_path}")


def load_documents() -> List[Dict]:
    """Load all documents from documents directory."""
    documents = []
    
    for doc_path in DOCUMENTS_DIR.glob("*.md"):
        with open(doc_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        chunks = chunk_text(content)
        
        documents.append({
            'path': str(doc_path.relative_to(LLM_KNOWLEDGE_DIR)),
            'content': content,
            'chunks': chunks,
            'hash': hashlib.sha256(content.encode()).hexdigest()
        })
    
    return documents


def main():
    """Build RAG index from documents."""
    print("=" * 80)
    print("RansomEye Intelligence - RAG Index Builder (Lightweight)")
    print("=" * 80)
    print()
    print("Building pre-indexed RAG knowledge base using TF-IDF...")
    print()
    
    # Load documents
    print("Loading documents...")
    documents = load_documents()
    print(f"  ✓ Loaded {len(documents)} documents")
    for doc in documents:
        print(f"    - {doc['path']}: {len(doc['chunks'])} chunks")
    print()
    
    # Build index
    print("Computing TF-IDF embeddings...")
    embeddings, chunks, vocabulary, idf_scores = compute_tfidf_embeddings(documents)
    print(f"  ✓ Generated {len(chunks)} embeddings")
    print(f"  ✓ Vocabulary size: {len(vocabulary)}")
    print(f"  ✓ Embedding dimension: {embeddings.shape[1]}")
    print()
    
    # Save index
    save_index_simple(embeddings, chunks, documents, vocabulary, idf_scores)
    print()
    
    print("=" * 80)
    print("✓ RAG index build complete")
    print("=" * 80)


if __name__ == '__main__':
    main()

