<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { listen } from '@tauri-apps/api/event';
  
    let transcript: string = '';
    let queue: string[] = [];
    let animating: boolean = false;
    let isRecording: boolean = false;
  
    function getTypingSpeed(): number {
      const baseSpeed = 50;
      const speedFactor = 5;
      return Math.max(20, baseSpeed - queue.length * speedFactor);
    }
  
    onMount(() => {
      listen('transcribe', (event) => {
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
      isRecording = true;
      await invoke('start_recording', {});
    }
  
    async function stop_recording() {
      isRecording = false;
      await invoke('stop_recording', {});
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
      display: inline-block;
      margin: 10px 10px;
      padding: 10px 20px;
      font-size: 1rem;
      border: none;
      border-radius: 50px;
      cursor: pointer;
      background: linear-gradient(45deg, #007BFF, #00BFFF);
      color: white;
      transition: background 0.3s, transform 0.2s;
      box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
    }
    button:hover {
      background: linear-gradient(45deg, #0056b3, #0095c7);
      transform: translateY(-2px);
    }
    .stop {
      background: linear-gradient(45deg, #dc3545, #e55353);
    }
    .audio-animation {
      display: flex;
      justify-content: center;
      align-items: flex-end;
      gap: 5px;
      margin: 20px auto;
      width: 100px;
      height: 50px;
    }
    .audio-animation .bar {
      width: 8px;
      background: #007BFF;
      animation: bounce 1s infinite;
    }
    .audio-animation .bar:nth-child(1) { animation-delay: 0s; }
    .audio-animation .bar:nth-child(2) { animation-delay: 0.1s; }
    .audio-animation .bar:nth-child(3) { animation-delay: 0.2s; }
    .audio-animation .bar:nth-child(4) { animation-delay: 0.3s; }
    .audio-animation .bar:nth-child(5) { animation-delay: 0.4s; }
  
    @keyframes bounce {
      0%, 100% { height: 10px; }
      50% { height: 50px; }
    }
  </style>
  
  <div>
    <button class="start" on:click={start_recording}>🎤 Start Recording</button>
    <button class="stop" on:click={stop_recording}>🙅‍♀️ Stop Recording</button>
  
    {#if isRecording}
      <div class="audio-animation">
        <div class="bar"></div>
        <div class="bar"></div>
        <div class="bar"></div>
        <div class="bar"></div>
        <div class="bar"></div>
      </div>
    {/if}
  
    <div class="machine-code">
      {transcript}
    </div>
  </div>
  