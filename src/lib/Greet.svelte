<script>
  import { invoke } from '@tauri-apps/api/core'

  let name = ''
  let greetMsg = ''

  async function greet() {
    console.log('Greeting', name)
    greetMsg = await invoke('greet', { name })
  }
</script>

<div>
  <input id="greet-input" placeholder="Enter a name..." bind:value="{name}" />
  <button on:click="{greet}">Greet</button>
  <p>{greetMsg}</p>
</div>