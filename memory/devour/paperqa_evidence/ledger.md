# Devour: PaperQA Evidence Ledger / Citation Grounding Pattern

## Objective

吞噬 Future-House `paper-qa` 的证据账本、引用绑定、上下文评分模式，用于补强 APEX 的“吞噬后必须有证据链”能力，避免只排名/只叙事。

## Source Evidence

- Repo: `git@github.com:Future-House/paper-qa.git`
- Commit: `d2c3c698fdf06986aa021812ab3186d3696438d8`
- Local snapshot: `vendor/github/Future-House/paper-qa`

Inspected files:

- `src/paperqa/types.py`
- `src/paperqa/core.py`
- `README.md`
- `docs/tutorials/querying_with_clinical_trials.md`

## Distilled Pattern

PaperQA centers answers around typed evidence objects:

1. **Doc** — source identity and citation.
   - fields include `docname`, `dockey`, `citation`, `content_hash`
   - CSV serialization excludes embeddings, which keeps manifests readable and model-agnostic.

2. **Text** — retrievable chunk linked to a source document.
   - `text`: processed text content
   - `name`: human-readable chunk id
   - `doc`: source `Doc` / `DocDetails`
   - optional media metadata is separated from actual text.

3. **Context** — question-conditioned evidence snippet.
   - `context`: summary relevant to a specific question
   - `question`: the question being answered
   - `score`: relevance score 0-10
   - auto-generated `pqac-*` ID allows answer references to map back to evidence.

4. **PQASession** — answer ledger.
   - keeps `question`, `answer`, `raw_answer`, `contexts`, `references`, `formatted_answer`
   - `used_contexts` computes which context IDs were actually cited in the raw answer
   - `populate_formatted_answers_and_bib_from_raw_answer()` replaces context IDs with citations and strips hallucinated citations not present in contexts.

5. **LLM JSON repair / retry boundary** (`core.py`).
   - `llm_parse_json` repairs common malformed JSON outputs and normalizes relevance scores.
   - context generation distinguishes retryable bad JSON from non-retryable timeout.

## APEX Adaptation

For APEX Devour, every distillation must produce an evidence ledger:

```text
DevourEvidence = {
  source_repo,
  commit,
  local_path,
  inspected_files,
  extracted_contexts: [ {id, quote_or_summary, source_path, score} ],
  used_context_ids,
  distilled_pattern,
  verification,
  hallucinated_claims_removed
}
```

Rules:

- A claim can enter Σ_memory only if it has a source path + commit + score.
- Any generated conclusion must cite at least one context id.
- Claims whose citation id does not map to a context are stripped or marked unverified.
- Embeddings are optional and excluded from durable manifests.
- Bad JSON is retryable; timeout is non-retryable and should be archived as Episodic failure.

## Σ_memory Injection

This devour contributes:

- **Working** memories: evidence-ledger execution rules and citation stripping workflow.
- **Semantic** memories: `Doc/Text/Context/PQASession` object roles.
- **Episodic** memories: LLM bad JSON vs timeout distinction as failure-handling pattern.

## Verification

- Repo cloned via SSH and commit recorded.
- Source files inspected locally.
- No secrets copied.
- Pattern distilled; no blind source copy.
