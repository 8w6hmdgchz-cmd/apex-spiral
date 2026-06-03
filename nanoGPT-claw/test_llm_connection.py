#!/usr/bin/env python3
"""
LLM Connection Test Script
Tests if the LLM calling chain is truly connected
"""

import os
import sys
import asyncio
import subprocess

def check_env_vars():
    """Check if LLM API keys are configured"""
    print("🔍 Checking LLM API configuration...")
    
    providers = []
    
    if os.getenv("OPENAI_API_KEY"):
        providers.append("✅ OpenAI")
    else:
        providers.append("❌ OpenAI (missing OPENAI_API_KEY)")
    
    if os.getenv("ANTHROPIC_API_KEY"):
        providers.append("✅ Anthropic")
    else:
        providers.append("❌ Anthropic (missing ANTHROPIC_API_KEY)")
    
    if os.getenv("OLLAMA_BASE_URL"):
        providers.append("✅ Ollama")
    else:
        providers.append("❌ Ollama (missing OLLAMA_BASE_URL)")
    
    if os.getenv("OPENAI_API_BASE") and os.getenv("OPENAI_API_MODEL"):
        providers.append("✅ Custom OpenAI-compatible")
    else:
        providers.append("❌ Custom OpenAI (need OPENAI_API_BASE + OPENAI_API_MODEL)")
    
    print("\n📡 Available providers:")
    for p in providers:
        print(f"  {p}")
    
    return any([
        os.getenv("OPENAI_API_KEY"),
        os.getenv("ANTHROPIC_API_KEY"),
        os.getenv("OLLAMA_BASE_URL"),
        (os.getenv("OPENAI_API_BASE") and os.getenv("OPENAI_API_MODEL"))
    ])

async def test_rust_binary():
    """Test if the Rust binary can run and respond"""
    print("\n🚀 Testing Rust binary...")
    
    try:
        result = subprocess.run(
            ["cargo", "run", "--", "status"],
            capture_output=True,
            text=True,
            timeout=30,
            cwd="/workspace/nanoGPT-claw"
        )
        
        if result.returncode == 0:
            print("✅ Rust binary runs successfully!")
            print(f"\nOutput:\n{result.stdout}")
            return True
        else:
            print("❌ Rust binary failed to run")
            print(f"Error:\n{result.stderr}")
            return False
    except Exception as e:
        print(f"❌ Error running binary: {e}")
        return False

def main():
    print("="*60)
    print("🧪 NanoGPT-Claw LLM Connection Test")
    print("="*60)
    
    # Check environment
    has_provider = check_env_vars()
    
    if not has_provider:
        print("\n⚠️  WARNING: No LLM providers configured!")
        print("\nTo enable LLM functionality, set one of:")
        print("  export OPENAI_API_KEY=your_key")
        print("  export ANTHROPIC_API_KEY=your_key")
        print("  export OLLAMA_BASE_URL=http://localhost:11434")
        print("  export OPENAI_API_BASE=https://api.example.com/v1")
        print("  export OPENAI_API_MODEL=model-name")
    
    # Test binary
    binary_works = asyncio.run(test_rust_binary())
    
    print("\n" + "="*60)
    if binary_works:
        print("✅ Binary test PASSED")
    else:
        print("❌ Binary test FAILED")
    print("="*60)
    
    return 0 if binary_works else 1

if __name__ == "__main__":
    sys.exit(main())
