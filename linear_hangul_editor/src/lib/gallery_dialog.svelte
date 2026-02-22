<script>
    // @ts-nocheck
    import { X } from "lucide-svelte";

    let {
        open = false,
        gallery_renderings = [],
        gallery_selected_rendering = 0,
        gallery_selected_config_data = "",
        gallery_selected_kerning_data = "",
        gallery_font_names = [],
        char_size = $bindable("16"),
        choose_gallery_rendering = () => {},
        set_gallery_rendering_font = () => {},
        close_gallery_dialog = () => {},
    } = $props();

    const selected_rendering = $derived(
        gallery_selected_rendering >= 0 &&
            gallery_selected_rendering < gallery_renderings.length
            ? gallery_renderings[gallery_selected_rendering]
            : null,
    );
</script>

{#if open}
    <div class="fixed inset-0 z-[57] flex items-center justify-center bg-black/50 p-3">
        <div class="ui-card flex h-[92vh] w-full max-w-[96rem] min-h-0 flex-col p-3">
            <div class="flex items-center justify-between gap-3 border-b border-[hsl(var(--border))] pb-3">
                <div class="text-sm font-semibold">Gallery</div>
                <button
                    class="ui-button-ghost"
                    onclick={close_gallery_dialog}
                    title="Close gallery dialog"
                    aria-label="Close gallery dialog"
                >
                    <X class="h-4 w-4" />
                </button>
            </div>

            <div class="grid min-h-0 flex-1 grid-cols-[18rem_minmax(0,1fr)] gap-3 pt-3">
                <div class="grid min-h-0 h-full grid-rows-[auto_minmax(0,1fr)_minmax(0,1fr)] gap-3">
                    <div class="ui-card p-3">
                        <div class="ui-label">Selected Font</div>
                        <div class="mt-1 text-xs font-semibold">
                            {selected_rendering?.font_name || "-"}
                        </div>
                        <div class="mt-1 text-xs text-[hsl(var(--muted-foreground))]">
                            {selected_rendering?.has_matching_toolset
                                ? "Matching toolset found"
                                : "No matching-name toolset"}
                        </div>
                    </div>
                    <div class="ui-card flex min-h-0 flex-col p-3">
                        <div class="ui-label mb-2">Kerning</div>
                        <textarea
                            class="h-full min-h-0 flex-1 resize-none rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--background))] p-3 font-mono text-xs"
                            readonly
                            value={gallery_selected_kerning_data}
                        ></textarea>
                    </div>
                    <div class="ui-card flex min-h-0 flex-col p-3">
                        <div class="ui-label mb-2">Config</div>
                        <textarea
                            class="h-full min-h-0 flex-1 resize-none rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--background))] p-3 font-mono text-xs"
                            readonly
                            value={gallery_selected_config_data}
                        ></textarea>
                    </div>
                </div>

                <div class="ui-card flex flex-col p-3 h-full min-h-full">
                    <div class="mb-2 flex items-center justify-between">
                        <div class="ui-label">Renderings</div>
                        <div class="text-xs text-[hsl(var(--muted-foreground))]">
                            {gallery_renderings.length} fonts
                        </div>
                    </div>
                    <div class="flex pr-1 h-full w-full flex-wrap overflow-auto">
                        {#each gallery_renderings as rendering, idx}
                            <div
                                class={`ui-card h-64 flex flex-col gap-2 p-2 ${
                                    gallery_selected_rendering == idx
                                        ? "border-[hsl(var(--primary))]"
                                        : ""
                                }`}
                                role="button"
                                tabindex="0"
                                onclick={() => choose_gallery_rendering(idx)}
                                onkeydown={(event) => {
                                    if (event.key == "Enter" || event.key == " ") {
                                        event.preventDefault();
                                        choose_gallery_rendering(idx);
                                    }
                                }}
                            >
                                <div class="flex items-center gap-2">
                                    <select
                                        class="ui-select h-8 min-w-0 flex-1 text-xs"
                                        value={rendering.font_name}
                                        onclick={(event) => event.stopPropagation()}
                                        onchange={(event) => {
                                            choose_gallery_rendering(idx);
                                            set_gallery_rendering_font(
                                                idx,
                                                event.currentTarget.value,
                                            );
                                        }}
                                    >
                                        {#each gallery_font_names as font_item}
                                            <option value={font_item}>{font_item}</option>
                                        {/each}
                                    </select>
                                </div>
                                {#key `${idx}-${rendering.render_version ?? 0}`}
                                    <textarea
                                        class="w-64 h-full resize-none rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--background))] p-2 linkor text-sm cursor-pointer"
                                        readonly
                                        value={rendering.text}
                                        style:font-family={`${rendering.font_family || "Linear Korean"}, "Noto Sans KR", sans-serif`}
                                        style:font-size={char_size + "px"}
                                    ></textarea>
                                {/key}
                            </div>
                        {/each}
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}
