<script>
    // @ts-nocheck
    import { RefreshCcw, Trash2, X } from "lucide-svelte";

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
        reorder_gallery_renderings = () => {},
        refresh_gallery_dialog = () => {},
        delete_selected_gallery_card_assets = () => {},
        close_gallery_dialog = () => {},
    } = $props();

    let dragging_index = $state(-1);
    let drag_over_index = $state(-1);
    let grabX = 0;
    let grabY = 0;

    function parse_drag_index(value, fallback = -1) {
        const parsed = Number(value);
        return Number.isInteger(parsed) ? parsed : fallback;
    }

    function get_card_index_from_event_target(target) {
        if (!(target instanceof Element)) {
            return -1;
        }
        const card = target.closest("[data-gallery-card-index]");
        if (!(card instanceof HTMLElement)) {
            return -1;
        }
        return parse_drag_index(card.dataset.galleryCardIndex, -1);
    }

    function find_drop_index_from_point(client_x, client_y) {
        if (!Number.isFinite(client_x) || !Number.isFinite(client_y)) {
            return -1;
        }
        const card_elements = document.querySelectorAll(
            "[data-gallery-card-index]",
        );
        for (const card of card_elements) {
            if (!(card instanceof HTMLElement)) {
                continue;
            }
            const rect = card.getBoundingClientRect();
            const is_inside =
                client_x >= rect.left &&
                client_x <= rect.right &&
                client_y >= rect.top &&
                client_y <= rect.bottom;
            if (!is_inside) {
                continue;
            }
            let idx = parse_drag_index(card.dataset.galleryCardIndex, -1);
            return idx;
        }
        return -1;
    }

    function handle_drag_end(event = null, allow_reorder = true) {
        if (allow_reorder && dragging_index >= 0) {
            let target_index = -1;
            if (event != null) {
                let x = event.clientX - grabX + 10;
                let y = window.innerHeight - event.clientY - grabY + 240;
                target_index = find_drop_index_from_point(
                    x,
                    y
                );
            }
            if (target_index >= 0 && target_index != dragging_index) {
                reorder_gallery_renderings(dragging_index, target_index);
            }
        }
        dragging_index = -1;
        drag_over_index = -1;
    }

    function handle_cards_drag_over(event) {
        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
        if (dragging_index < 0) {
            return;
        }
        const target_index = get_card_index_from_event_target(event.target);
        if (target_index >= 0) {
            drag_over_index = target_index;
        }
    }

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
                <div class="flex items-center gap-2">
                    <select class="ui-select h-8 min-w-[5.5rem]" bind:value={char_size}>
                        <option value="16">16</option>
                        <option value="24">24</option>
                        <option value="48">48</option>
                        <option value="64">64</option>
                    </select>
                    <button
                        class="ui-button-ghost"
                        onclick={delete_selected_gallery_card_assets}
                        title="Delete selected card font and toolset"
                        aria-label="Delete selected card font and toolset"
                    >
                        <Trash2 class="h-4 w-4" />
                    </button>
                    <button
                        class="ui-button-ghost"
                        onclick={refresh_gallery_dialog}
                        title="Refresh gallery"
                        aria-label="Refresh gallery"
                    >
                        <RefreshCcw class="h-4 w-4" />
                    </button>
                    <button
                        class="ui-button-ghost"
                        onclick={close_gallery_dialog}
                        title="Close gallery dialog"
                        aria-label="Close gallery dialog"
                    >
                        <X class="h-4 w-4" />
                    </button>
                </div>
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
                    <div
                        class="flex pr-1 h-full w-full flex-wrap overflow-auto relative"
                        role="list"
                        aria-label="Gallery renderings"
                    >
                        {#each gallery_renderings as rendering, idx}
                            <div
                                class={`ui-card h-64 flex flex-col gap-2 p-2 ${
                                    gallery_selected_rendering == idx
                                        ? "border-[hsl(var(--primary))]"
                                        : ""
                                } ${
                                    drag_over_index == idx && dragging_index != idx
                                        ? "ring-1 ring-[hsl(var(--primary))]"
                                        : ""
                                }`}
                                role="button"
                                tabindex="0"
                                draggable="true"
                                dropzone="move"
                                data-gallery-card-index={idx}
                                onclick={() => choose_gallery_rendering(idx)}
                                ondragstart={(event) => {
                                    const rect = event.target.getBoundingClientRect();
                                    grabX = event.clientX - rect.left;
                                    grabY = event.clientY - rect.top;
                                    dragging_index = idx;
                                    drag_over_index = idx;
                                    event.dataTransfer.effectAllowed = "move";
                                    event.dataTransfer.setData(
                                        "text/plain",
                                        String(idx),
                                    );
                                }}
                                ondragend={handle_drag_end}
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
                                        draggable="false"
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
                                        draggable="false"
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
