<script lang="ts">
    import {onMount} from "svelte";

    let legend: string[] = JSON.parse(atob(`%%legend%%`))

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

<main class="min-h-dvh w-full flex justify-center py-4 sm:py-16 px-4">
    <div class="w-full max-w-4xl">
        <div class="w-full grid grid-cols-5 gap-8 sm:gap-16">
            <div class="col-span-full {legend?.length ? 'sm:col-span-4' : ''}">
                {@html atob('%%content%%')}
            </div>

            {#if legend?.length}
                <div class="flex-col hidden sm:flex sticky top-8 h-fit">
                    <p class="text-slate-900 font-bold pb-2 text-lg">Legend</p>

                    {#each legend as page}
                        <a href="{page}" class="text-slate-900 hover:text-blue-600">
                            {page}
                        </a>
                    {/each}
                </div>
            {/if}
        </div>
        <div class="h-32"></div>
    </div>
</main>

<style>
    @import 'tailwindcss';

    :global(body) {
        @apply bg-slate-50;
    }

    :global(h1, h2, h3, h4, h5, h6) {
        @apply mt-8 mb-2 text-slate-900 font-bold border-b border-slate-200;
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
        @apply border-l-4 border-slate-200 pl-4 m-0;
    }

    :global(img) {
        @apply max-w-full rounded;
    }
</style>