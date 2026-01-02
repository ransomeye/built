-- Path and File Name : /home/ransomeye/rebuild/core/ai/models/create_inference_schema.sql
-- Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
-- Details of functionality of this file: Database schema for AI inference results with SHAP explainability

-- AI Inference Results Table
CREATE TABLE IF NOT EXISTS ai_inference_results (
    inference_id VARCHAR(36) PRIMARY KEY,
    model_name VARCHAR(255) NOT NULL,
    model_version VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Inference inputs
    input_features JSONB NOT NULL,
    feature_names JSONB NOT NULL,
    
    -- Inference outputs
    confidence_score DOUBLE PRECISION NOT NULL,
    calibrated_confidence DOUBLE PRECISION NOT NULL,
    risk_score DOUBLE PRECISION NOT NULL,
    recommendation TEXT NOT NULL,
    
    -- SHAP explainability (MANDATORY)
    shap_baseline_value DOUBLE PRECISION NOT NULL,
    shap_values JSONB NOT NULL,
    feature_contributions JSONB NOT NULL,
    shap_explanation_hash VARCHAR(64) NOT NULL,
    
    -- Model metadata
    model_hash VARCHAR(64) NOT NULL,
    model_signature TEXT,
    
    -- Audit fields
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for query performance
CREATE INDEX IF NOT EXISTS idx_ai_inference_model_name ON ai_inference_results(model_name);
CREATE INDEX IF NOT EXISTS idx_ai_inference_timestamp ON ai_inference_results(timestamp);
CREATE INDEX IF NOT EXISTS idx_ai_inference_risk_score ON ai_inference_results(risk_score);
CREATE INDEX IF NOT EXISTS idx_ai_inference_model_hash ON ai_inference_results(model_hash);

-- Comments
COMMENT ON TABLE ai_inference_results IS 'AI/ML inference results with mandatory SHAP explainability';
COMMENT ON COLUMN ai_inference_results.shap_values IS 'SHAP values for each feature (MANDATORY for explainability)';
COMMENT ON COLUMN ai_inference_results.feature_contributions IS 'Feature contribution breakdown (MANDATORY for explainability)';
COMMENT ON COLUMN ai_inference_results.shap_explanation_hash IS 'SHA256 hash of SHAP explanation for integrity verification';

