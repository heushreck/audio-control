<script>
  import Transcribe from '../lib/Transcribe.svelte';
</script>

<div class="container">
  <h1>Local Rust Transcription App</h1>
  <Transcribe />
</div>

<style>
  :global(body) {
    background-color: #333333;
    color: #ffffff;
    margin: 0;
    padding: 0;
    font-family: 'SF Pro Display', 'Segoe UI', system-ui, -apple-system, sans-serif;
  }
  
  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
    text-align: center;
  }
  
  h1 {
    font-size: 2.5rem;
    margin-bottom: 2rem;
    font-weight: 600;
    color: #f0f0f0;
    letter-spacing: -0.5px;
  }
</style>
