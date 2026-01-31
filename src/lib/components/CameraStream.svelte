<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core"; // or @tauri-apps/api/tauri in v1
    import JMuxer from "jmuxer";

    let videoElement: HTMLVideoElement;
    let jmuxer: any;
    let unlisten: () => void;

    onMount(async () => {
        // Initialize JMuxer attached to the video element
        jmuxer = new JMuxer({
            node: videoElement,
            mode: "video",
            flushingTime: 0, // CRITICAL: Tell JMuxer to push to the video element immediately
            // maxDelay: 100, // If the delay exceeds 100ms, drop old frames
            fps: 30,
            debug: false,
        });

        // Listen for binary data from Rust
        unlisten = await listen<number[]>("video-packet", (event) => {
            // console.log("Received bytes:", event.payload.length); // Should see numbers like 4096, 16384
            // event.payload is the raw byte array
            // JMuxer expects a Uint8Array
            const data = new Uint8Array(event.payload);

            // Feed the H.264 chunk to the muxer
            jmuxer.feed({
                video: data,
            });
        });

        // Tell Rust to start the process
        invoke("start_camera_stream");
    });

    onDestroy(() => {
        if (unlisten) unlisten();
        if (jmuxer) jmuxer.destroy();
    });
</script>

<video bind:this={videoElement} autoplay muted playsinline controls> </video>

<style>
    video {
        width: 100%;
        max-width: 800px;
        background: black;
    }
</style>
