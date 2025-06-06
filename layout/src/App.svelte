<script lang="ts">
    import {onMount} from "svelte";

    let legend = $state<string[]>(JSON.parse(atob(`%%legend%%`)));
    let darkMode = $state<boolean>();

    onMount(
        () =>
            (darkMode = localStorage.getItem('theme')
                ? localStorage.getItem('theme') === 'dark'
                : window.matchMedia('(prefers-color-scheme: dark)').matches),
    );

    $effect(() => {
        if (darkMode === undefined) return;
        localStorage.setItem('theme', darkMode ? 'dark' : 'light');
        if (darkMode) document.documentElement.classList.add('dark');
        else document.documentElement.classList.remove('dark');
    });

    onMount(() => {
        const ws = new WebSocket(`ws://${window.location.host}/ws`);
        console.log("Connected to websocket");

        ws.addEventListener("message", (message) => {
            const {event, data} = JSON.parse(message.data);

            if (event === "pageupdate" && data.page === "/%%page%%") {
                location.reload();
            }

            if (event === "legendupdate") {
                legend = JSON.parse(atob(data.legend));
            }
        });
    })
</script>

<svelte:head>
    <script>
        let theme = localStorage.getItem('theme');

        if (!theme) {
            const prefersDark = window.matchMedia(
                '(prefers-color-scheme: dark)',
            ).matches;
            theme = prefersDark ? 'dark' : 'light';
            localStorage.setItem('theme', theme);
        }

        document.documentElement.classList.toggle('dark', theme === 'dark');
    </script>
</svelte:head>

<main class="min-h-dvh w-full flex justify-center py-4 sm:py-16 px-4">
    <div class="w-full max-w-5xl">
        <div class="w-full grid grid-cols-4 gap-8 sm:gap-16">
            <div class="col-span-full {legend?.length ? 'sm:col-span-3' : ''}">
                {@html atob('%%content%%')}
            </div>

            {#if legend?.length}
                <div class="flex-col hidden sm:flex sticky py-8 top-0 h-fit col-span-1">
                    <p class="text-slate-900 font-bold pb-2 text-xl dark:text-slate-100">Legend</p>

                    {#each legend as page}
                        <a href="{page}" class="text-slate-900 hover:text-blue-600 dark:text-slate-100 font-mono">
                            {page}
                        </a>
                    {/each}

                    <button
                            onclick={() => (darkMode = !darkMode)}
                            class="text-slate-900 hover:text-blue-600 dark:text-slate-100 font-mono text-left cursor-pointer mt-8"
                    >
                        Toggle dark mode
                    </button>
                </div>
            {/if}
        </div>
        <div class="h-32"></div>
    </div>
</main>

<style>
    @import 'tailwindcss';

    @custom-variant dark (&:where(.dark, .dark *));

    :global(body) {
        @apply bg-slate-50 dark:bg-slate-900 text-slate-900 dark:text-slate-100 font-sans;
    }

    :global(h1, h2, h3, h4, h5, h6) {
        @apply mt-8 mb-2 text-slate-900 font-bold border-b border-slate-200 dark:border-slate-800 dark:text-slate-100;
    }

    :global(h1) {
        @apply text-4xl;
    }

    :global(h2) {
        @apply text-3xl;
    }

    :global(h3) {
        @apply text-2xl;
    }

    :global(h4) {
        @apply text-xl;
    }

    :global(h5) {
        @apply text-lg;
    }

    :global(h6) {
        @apply text-base;
    }

    :global(blockquote) {
        @apply border-l-4 border-slate-200 pl-4 m-0 dark:border-slate-800;
    }

    :global(img) {
        @apply max-w-full rounded;
    }

    :global(a) {
        @apply text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-600;
    }

    :global(pre) {
        @apply overflow-x-auto bg-slate-200 p-4 rounded dark:bg-slate-800;
    }
</style>