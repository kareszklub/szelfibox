<script>
	import './layout.css';

	let { children } = $props();

	import "../app.css";
</script>

<main class="flex flex-col h-[100vh]">
	<header class="flex justify-center"><h1>Szelfibox</h1></header>
	<div class="flex-grow flex flex-col justify-center">
		{@render children()}
	</div>
	<footer>© 2025 Kareszklub Inc.</footer>
</main>
