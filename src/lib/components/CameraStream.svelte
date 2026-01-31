<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core"; // or @tauri-apps/api/tauri in v1
    import JMuxer from "jmuxer";

    let videoElement: HTMLVideoElement;
    let jmuxer: any;
    let unlisten: () => void;

    /**
     * Captures the current frame and returns both raw ImageData and a displayable Blob URL.
     */
    export async function getImageData(): Promise<{
        imageData: ImageData;
        blobUrl: string;
    } | null> {
        if (!videoElement || videoElement.videoWidth === 0) return null;

        // 1. Create a temporary offscreen canvas
        const canvas = document.createElement("canvas");
        canvas.width = videoElement.videoWidth;
        canvas.height = videoElement.videoHeight;

        const ctx = canvas.getContext("2d", { willReadFrequently: true });
        if (!ctx) return null;

        // 2. Draw the current video frame onto the canvas
        ctx.drawImage(videoElement, 0, 0, canvas.width, canvas.height);

        // 3. Extract the pixel data (synchronous)
        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

        // 4. Create the Blob URL (asynchronous)
        const blobUrl = await new Promise<string | null>((resolve) => {
            canvas.toBlob(
                (blob) => {
                    if (blob) {
                        resolve(URL.createObjectURL(blob));
                    } else {
                        resolve(null);
                    }
                },
                "image/jpeg",
                1.0,
            );
        });

        if (!blobUrl) return null;

        console.log("Done with the image creation and Blob URL generation");

        return { imageData, blobUrl };
    }

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
