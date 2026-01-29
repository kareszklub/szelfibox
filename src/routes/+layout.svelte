<script>
    import "../app.css";
    import { Separator } from "$lib/components/ui/separator";

    let { children } = $props();
</script>

<div class="min-h-screen bg-background font-sans text-foreground flex flex-col">
    <header class="py-6">
        <div class="container mx-auto px-4 flex justify-center items-center">
            <h1 class="text-3xl font-bold tracking-tight">📸 Szelfibox</h1>
        </div>
    </header>

    <div class="container mx-auto px-4 mb-6">
        <Separator />
    </div>

    <main
        class="flex-grow flex flex-col justify-center items-center container mx-auto px-4"
    >
        {@render children()}
    </main>

    <footer class="py-6 text-center text-sm text-muted-foreground">
        <p>© 2026 Kareszklub Inc.</p>
    </footer>
</div>
