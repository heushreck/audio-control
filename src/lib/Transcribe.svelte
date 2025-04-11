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
  
    async function toggleRecording() {
      if (isRecording) {
        await stopRecording();
      } else {
        await startRecording();
      }
    }

    async function startRecording() {
      isRecording = true;
      await invoke('start_recording', {});
    }
  
    async function stopRecording() {
      isRecording = false;
      await invoke('stop_recording', {});
    }
  
    onDestroy(() => {
      // Add cleanup logic if needed
    });
  </script>
  
  <style>
    .transcription-container {
      display: flex;
      flex-direction: column;
      align-items: center;
      margin-top: 2rem;
    }
    
    .controls-container {
      display: flex;
      flex-direction: column;
      align-items: center;
      margin-bottom: 2rem;
      min-height: 140px;
    }
    
    .record-button {
      width: 64px;
      height: 64px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      background: linear-gradient(135deg, #ff4d4d, #f9615a);
      box-shadow: 0 4px 20px rgba(249, 97, 90, 0.5);
      border: none;
      transition: transform 0.2s ease, box-shadow 0.2s ease;
      z-index: 2;
    }
    
    .record-button:hover {
      transform: translateY(-2px);
      box-shadow: 0 6px 24px rgba(249, 97, 90, 0.6);
    }
    
    .record-button.recording {
      background: linear-gradient(135deg, #6e6e6e, #4a4a4a);
      box-shadow: 0 4px 20px rgba(74, 74, 74, 0.5);
    }
    
    .record-icon {
      font-size: 24px;
      color: white;
    }
    
    .transcript-box {
      width: 100%;
      font-family: 'Menlo', 'Consolas', monospace;
      background-color: #424242;
      color: #f0f0f0;
      padding: 2rem;
      border-radius: 10px;
      white-space: pre-wrap;
      min-height: 200px;
      box-shadow: 0 8px 30px rgba(0, 0, 0, 0.15);
      border-left: 4px solid #ff4d4d;
      line-height: 1.6;
      text-align: left;
    }

    /* Wave Animation */
    .wave-container {
      height: 60px;
      width: 100%;
      display: flex;
      align-items: center;
      justify-content: center;
      margin-top: 1rem;
    }
    
    .wave {
      display: flex;
      align-items: center;
      gap: 4px;
    }
    
    .wave-bar {
      background: linear-gradient(to top, #ff4d4d, #f9615a);
      width: 4px;
      border-radius: 2px;
      transform-origin: bottom;
    }
    
    @keyframes wave {
      0%, 100% {
        height: 10px;
      }
      50% {
        height: 40px;
      }
    }
    
    .wave-bar:nth-child(1) { animation: wave 1s ease-in-out infinite; animation-delay: 0s; }
    .wave-bar:nth-child(2) { animation: wave 1s ease-in-out infinite; animation-delay: 0.1s; }
    .wave-bar:nth-child(3) { animation: wave 1s ease-in-out infinite; animation-delay: 0.2s; }
    .wave-bar:nth-child(4) { animation: wave 1s ease-in-out infinite; animation-delay: 0.3s; }
    .wave-bar:nth-child(5) { animation: wave 1s ease-in-out infinite; animation-delay: 0.4s; }
    .wave-bar:nth-child(6) { animation: wave 1s ease-in-out infinite; animation-delay: 0.5s; }
    .wave-bar:nth-child(7) { animation: wave 1s ease-in-out infinite; animation-delay: 0.6s; }
    .wave-bar:nth-child(8) { animation: wave 1s ease-in-out infinite; animation-delay: 0.5s; }
    .wave-bar:nth-child(9) { animation: wave 1s ease-in-out infinite; animation-delay: 0.4s; }
    .wave-bar:nth-child(10) { animation: wave 1s ease-in-out infinite; animation-delay: 0.3s; }
    .wave-bar:nth-child(11) { animation: wave 1s ease-in-out infinite; animation-delay: 0.2s; }
    .wave-bar:nth-child(12) { animation: wave 1s ease-in-out infinite; animation-delay: 0.1s; }
  </style>
  
  <div class="transcription-container">
    <div class="controls-container">
      <button class="record-button {isRecording ? 'recording' : ''}" on:click={toggleRecording}>
        <span class="record-icon">{isRecording ? '■' : '🎤'}</span>
      </button>
      
      {#if isRecording}
        <div class="wave-container">
          <div class="wave">
            {#each Array(12) as _, i}
              <div class="wave-bar"></div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
    
    <div class="transcript-box">
      {transcript}
    </div>
  </div>
  