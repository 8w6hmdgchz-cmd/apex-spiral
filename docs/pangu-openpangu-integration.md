# Pangu / OpenPangu Integration Note

Generated: 2026-05-23 15:39:54 +0800

## What was found

The current A2A cache contains vLLM OpenPangu support files under:

`a2a-resources/cache/vllm-project_vllm/`

No standalone `pangu` folder was found. The value is inside the absorbed vLLM repository.

## Capability Evidence

- L462: | `PanguEmbeddedForCausalLM` | openPangu-Embedded-7B | `FreedomIntelligence/openPangu-Embedded-7B-V1.1` | ✅︎ | ✅︎ |
- L463: | `PanguProMoEV2ForCausalLM` | openpangu-pro-moe-v2 | | ✅︎ | ✅︎ |
- L464: | `PanguUltraMoEForCausalLM` | openpangu-ultra-moe-718b-model | `FreedomIntelligence/openPangu-Ultra-MoE-718B-V1.1` | ✅︎ | ✅︎ |
- L607: | `OpenPanguVLForConditionalGeneration` | openpangu-VL | T + I<sup>E+</sup> + V<sup>E+</sup> | `FreedomIntelligence/openPangu-VL-7B` | ✅︎ | ✅︎ |

## Indexed Files

- `a2a-resources/cache/vllm-project_vllm/vllm/model_executor/models/openpangu.py` sha256 `b84becfa1b460dd1...`
- `a2a-resources/cache/vllm-project_vllm/vllm/model_executor/models/openpangu_mtp.py` sha256 `112b2363a3095666...`
- `a2a-resources/cache/vllm-project_vllm/vllm/model_executor/models/openpangu_vl.py` sha256 `bbb15e9763be1408...`
- `a2a-resources/cache/vllm-project_vllm/vllm/model_executor/models/registry.py` sha256 `6232508eb1c32f1f...`
- `a2a-resources/cache/vllm-project_vllm/tests/models/registry.py` sha256 `c0dcb7b579412437...`
- `a2a-resources/cache/vllm-project_vllm/docs/models/supported_models.md` sha256 `9ac8fd1a3be65380...`

## Integration Boundary

This is a **capability ledger/index integration** only:

- no Pangu weights downloaded
- no model server started
- no external API call performed
- no claim that local Pangu inference is ready

## Integrated Artifact

- `research/apex/ledgers/pangu-vllm-capability-card.json`

## Practical Value

- local evidence for OpenPangu support in vLLM
- model architecture knowledge for SearchSkill
- A2A tag: `vllm-project_vllm` contains Pangu/OpenPangu runtime support
- future route: if user provides weights/runtime, vLLM can be checked as a serving path
