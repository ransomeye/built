# RAG Usage

**Path and File Name:** `/home/ransomeye/rebuild/ransomeye_ai_advisory/docs/rag_usage.md`  
**Author:** nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU  
**Details:** RAG (Retrieval-Augmented Generation) usage and constraints

## Read-Only Index

- Index is read-only
- Pre-indexed documents only
- No external calls
- No index modification

## Deterministic Retrieval

- Retrieval is deterministic
- Same query → same results
- No randomness
- Reproducible outputs

## Integrity Verification

- Index integrity verified on load
- Hash verification
- Metadata validation
- Document count verification

## Usage Constraints

- No external API calls
- No dynamic indexing
- No write operations
- Bounded memory usage

