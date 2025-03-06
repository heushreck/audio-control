<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { listen } from "@tauri-apps/api/event";
  
    let transcript: string = '';
    let queue: string[] = [];
    let animating: boolean = false;

    function getTypingSpeed(): number {
        const baseSpeed = 50;
        const speedFactor = 5;
        return Math.max(20, baseSpeed - queue.length * speedFactor);
    }
  
    onMount(() => {
      listen("transcribe", (event) => {
        queue.push(event.payload as string);
        if (!animating) {
          animateNext();
        }
      }).then((unlisten) => {
        return () => unlisten();
      });
    });
  
    function animateNext(): void {
      if (queue.length === 0) return;
      animating = true;
      const text: string = queue.shift()!;
      let i: number = 0;
      const typingSpeed = getTypingSpeed();
      const typeInterval = setInterval(() => {
        transcript += text[i];
        i++;
        if (i === text.length) {
          clearInterval(typeInterval);
          animating = false;
          animateNext();
        }
      }, typingSpeed);
    }
  
    async function start_recording() {
      await invoke('start_recording', { });
    }

    async function stop_recording() {
      await invoke('stop_recording', { });
    }
  
    onDestroy(() => {
      // Add cleanup logic if needed
    });
  </script>
  
  <style>
    .machine-code {
        font-family: 'Courier New', Courier, monospace;
        background-color: #fff;
        color: rgb(1, 6, 20);
        padding: 10px;
        border: 1px solid rgb(1, 6, 20);
        white-space: pre-wrap;
        min-height: 150px;
        margin: 0 20px;
    }
    button {
        display: block;
        margin: 10px auto;
        padding: 10px 20px;
        font-size: 1rem;
        border: none;
        border-radius: 50px;
        cursor: pointer;
        background-color: #007BFF;
        color: white;
        transition: background-color 0.3s;
    }

    .stop {
        background-color: #dc3545;
    }

    button:hover {
        background-color: #0056b3;
    }
  </style>
  
  <div>
    <button class="start" on:click={start_recording}>🎤 Start Recording</button>
    <button class="stop" on:click={stop_recording}>🙅‍♀️ Stop Recording</button>
    <div class="machine-code">
      {transcript}
    </div>
  </div>
  