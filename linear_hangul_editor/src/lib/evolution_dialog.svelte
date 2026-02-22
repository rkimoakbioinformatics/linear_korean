<script>
    // @ts-nocheck
    import {
        Copy,
        CopyPlus,
        Pencil,
        RotateCcw,
        Save,
        Shapes,
        Sparkles,
        Trash2,
        X,
    } from "lucide-svelte";

    const EVOLUTION_CONFIG_SHAPE = `{
  version: 1,
  kerning: {
    cho: {
      cho: {
        active: true,
        mutation_number: { min: -20, max: 20, step: 1 },
        include_prev: [],
        include_next: [],
        include_pairs: [],
        exclude_prev: [],
        exclude_next: [],
        exclude_pairs: [],
      },
    },
    jung: {},
    jong: {},
  },
  config: {
    source: {
      type: "string",
      mutation_string: { options: ["", "/absolute/path/to/source.ttf"] },
    },
    cho_type: {
      type: "string",
      mutation_string: { options: ["upperdot", "underdot", ""] },
    },
    jung_type: {
      type: "string",
      mutation_string: { options: ["upperdot", "underdot", ""] },
    },
    jong_type: {
      type: "string",
      mutation_string: { options: ["upperdot", "underdot", ""] },
    },
    cho_h_ratio: { type: "float", mutation_number: { min: 0.8, max: 1.2, step: 0.01 } },
    jung_w_ratio: { type: "float", mutation_number: { min: 0.8, max: 1.2, step: 0.01 } },
    jong_w_ratio: { type: "float", mutation_number: { min: 0.8, max: 1.2, step: 0.01 } },
    jung_h_ratio: { type: "float", mutation_number: { min: 0.8, max: 1.2, step: 0.01 } },
    jong_h_ratio: { type: "float", mutation_number: { min: 0.8, max: 1.2, step: 0.01 } },
    char_gap: { type: "float", mutation_number: { min: 0, max: 80, step: 1 } },
    cho_cho_gap: { type: "float", mutation_number: { min: 0, max: 80, step: 1 } },
    jung_jung_gap: { type: "float", mutation_number: { min: 0, max: 80, step: 1 } },
    jong_jong_gap: { type: "float", mutation_number: { min: 0, max: 80, step: 1 } },
    cho_jung_gap: { type: "float", mutation_number: { min: 0, max: 80, step: 1 } },
    jung_jong_gap: { type: "float", mutation_number: { min: 0, max: 80, step: 1 } },
    x_sw: { type: "float", mutation_number: { min: 0.8, max: 1.2, step: 0.01 } },
    y_sw: { type: "float", mutation_number: { min: 0.8, max: 1.2, step: 0.01 } },
    text_size: { type: "float", mutation_number: { min: 10, max: 48, step: 1 } },
    underdot_y: { type: "float", mutation_number: { min: -600, max: 200, step: 10 } },
    underdot_r_ratio: {
      type: "float",
      mutation_number: { min: 0.8, max: 1.2, step: 0.01 },
    },
    upperdot_y: { type: "float", mutation_number: { min: 1200, max: 2100, step: 10 } },
    upperdot_r_ratio: {
      type: "float",
      mutation_number: { min: 0.8, max: 1.2, step: 0.01 },
    },
    glyph_width: { type: "float", mutation_number: { min: 600, max: 1200, step: 10 } },
    cap_height: { type: "float", mutation_number: { min: 1400, max: 2200, step: 10 } },
    x_height: { type: "float", mutation_number: { min: 1000, max: 1900, step: 10 } },
    baseline: { type: "float", mutation_number: { min: -200, max: 200, step: 10 } },
    min_gap: { type: "float", mutation_number: { min: 0, max: 500, step: 10 } },
    space_width: { type: "float", mutation_number: { min: 200, max: 1200, step: 10 } },
    space_width_ratio: {
      type: "float",
      mutation_number: { min: 0.8, max: 1.2, step: 0.01 },
    },
  },
}
`;

    let {
        open = false,
        evolution_config_names = [],
        evolution_config_name = $bindable(""),
        evolution_config_data = $bindable(""),
        evolution_generation = 0,
        evolution_base_rendering = 0,
        evolution_selected_rendering = 0,
        evolution_renderings = [],
        evolution_selected_config_data = $bindable(""),
        evolution_selected_kerning_data = $bindable(""),
        char_size = "16",
        collision_check_enabled = $bindable(true),
        get_evolution_config_data = () => {},
        save_evolution_config_same_name = () => {},
        save_evolution_config_as = () => {},
        rename_evolution_config = () => {},
        delete_evolution_config = () => {},
        evolve_shell = () => {},
        evolve_from_rendering = () => {},
        render_again_from_rendering = () => {},
        reset_evolution_renderings = () => {},
        evolve_disabled = false,
        close_evolution_dialog = () => {},
        cache_current_evolution_draft = () => {},
        cache_selected_variant_draft = () => {},
        choose_evolution_rendering = () => {},
        copy_selected_variant_to_main_editors = () => {},
        replace_rendering_from_base = () => {},
        save_selected_variant_as = () => {},
    } = $props();

    let evolution_shape_dialog_open = $state(false);

    $effect(() => {
        if (!open) {
            evolution_shape_dialog_open = false;
        }
    });
</script>

{#if open}
    <div class="fixed inset-0 z-[55] flex items-center justify-center bg-black/50 p-3">
        <div class="ui-card flex h-[92vh] w-full max-w-[96rem] min-h-0 flex-col p-3">
            <div class="flex flex-wrap items-end justify-between gap-3 border-b border-[hsl(var(--border))] pb-3">
                <div class="space-y-1">
                    <div class="ui-label">Evolution Config</div>
                    <select
                        class="ui-select min-w-[14rem]"
                        bind:value={evolution_config_name}
                        onchange={get_evolution_config_data}
                    >
                        {#each evolution_config_names as item}
                            <option value={item}>{item}</option>
                        {/each}
                    </select>
                </div>
                <div class="flex flex-wrap items-center gap-2">
                    <button
                        class="ui-button-secondary"
                        onclick={save_evolution_config_same_name}
                        title="Save evolution config"
                        aria-label="Save evolution config"
                    >
                        <Save class="h-4 w-4" />
                    </button>
                    <button
                        class="ui-button-ghost"
                        onclick={rename_evolution_config}
                        title="Rename evolution config"
                        aria-label="Rename evolution config"
                    >
                        <Pencil class="h-4 w-4" />
                    </button>
                    <button
                        class="ui-button-ghost"
                        onclick={save_evolution_config_as}
                        title="Save evolution config as"
                        aria-label="Save evolution config as"
                    >
                        <CopyPlus class="h-4 w-4" />
                    </button>
                    <button
                        class="ui-button-ghost"
                        onclick={delete_evolution_config}
                        title="Delete evolution config"
                        aria-label="Delete evolution config"
                    >
                        <Trash2 class="h-4 w-4" />
                    </button>
                    <label class="ui-label flex items-center gap-1 rounded-md border border-[hsl(var(--border))] px-2 py-1">
                        <input type="checkbox" bind:checked={collision_check_enabled} />
                        Collision
                    </label>
                    <button
                        class="ui-button-ghost"
                        onclick={() => (evolution_shape_dialog_open = true)}
                        title="Show evolution config shape"
                        aria-label="Show evolution config shape"
                    >
                        <Shapes class="h-4 w-4" />
                    </button>
                    <button
                        class="ui-button-secondary"
                        onclick={reset_evolution_renderings}
                        disabled={evolve_disabled}
                        title="Reset evolution renderings"
                        aria-label="Reset evolution renderings"
                    >
                        <RotateCcw class="h-4 w-4" />
                    </button>
                    <button
                        class="ui-button-secondary"
                        onclick={copy_selected_variant_to_main_editors}
                        disabled={evolve_disabled}
                        title="Copy selected variant to main editors"
                        aria-label="Copy selected variant to main editors"
                    >
                        <Copy class="h-4 w-4" />
                    </button>
                    <button
                        class="ui-button-ghost"
                        onclick={save_selected_variant_as}
                        disabled={evolve_disabled}
                        title="Save selected variant as"
                        aria-label="Save selected variant as"
                    >
                        <CopyPlus class="h-4 w-4" />
                    </button>
                    <button
                        class="ui-button-primary"
                        onclick={() => evolve_shell()}
                        disabled={evolve_disabled}
                        title="Evolve"
                        aria-label="Evolve"
                    >
                        <Sparkles class="h-4 w-4" />
                    </button>
                    <button
                        class="ui-button-ghost"
                        onclick={close_evolution_dialog}
                        title="Close evolution dialog"
                        aria-label="Close evolution dialog"
                    >
                        <X class="h-4 w-4" />
                    </button>
                </div>
            </div>

            <div class="grid min-h-0 flex-1 grid-cols-1 gap-3 pt-3 xl:grid-cols-[16rem_1fr]">
                <div class="grid h-full gap-3">
                    <div class="ui-card flex min-h-0 flex-col p-3">
                        <div class="ui-label mb-2">Evolution Config Editor</div>
                        <textarea
                            class="h-full min-h-0 flex-1 resize-none rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--background))] p-3 font-mono text-xs"
                            bind:value={evolution_config_data}
                            oninput={cache_current_evolution_draft}
                        ></textarea>
                    </div>
                    <div class="ui-card flex min-h-0 flex-col p-3">
                        <div class="ui-label mb-2">Variant Kerning</div>
                        <textarea
                            class="h-44 min-h-0 flex-1 resize-none rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--background))] p-3 font-mono text-xs"
                            bind:value={evolution_selected_kerning_data}
                            oninput={(event) => cache_selected_variant_draft("kerning", event)}
                        ></textarea>
                    </div>
                    <div class="ui-card flex min-h-0 flex-col p-3">
                        <div class="ui-label mb-2">Variant Config</div>
                        <textarea
                            class="h-44 min-h-0 flex-1 resize-none rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--background))] p-3 font-mono text-xs"
                            bind:value={evolution_selected_config_data}
                            oninput={(event) => cache_selected_variant_draft("config", event)}
                        ></textarea>
                    </div>
                </div>

                <div class="ui-card flex flex-col p-3">
                    <div class="mb-2 flex items-center justify-between">
                        <div class="ui-label">Renderings</div>
                        <div class="text-xs text-[hsl(var(--muted-foreground))]">Generation {evolution_generation}</div>
                    </div>
                    <div class="grid min-h-0 flex-1 grid-cols-1 gap-2 overflow-auto pr-1 md:grid-cols-2">
                        {#each evolution_renderings as rendering, idx}
                            <div class="flex min-h-0 flex-col gap-1">
                                <div class="group relative">
                                    <button
                                        class={`ui-button-secondary h-8 w-full justify-start px-2 ${
                                            rendering?.is_system_font ? "pr-2" : "pr-44"
                                        } text-xs ${
                                            evolution_selected_rendering == idx
                                                ? "border-[hsl(var(--primary))] bg-[hsl(var(--accent))]"
                                                : ""
                                        } ${
                                            evolution_base_rendering == idx
                                                ? "ring-1 ring-[hsl(var(--primary))]"
                                                : ""
                                        }`}
                                        onclick={() => choose_evolution_rendering(idx)}
                                    >
                                        {rendering.label}
                                        {#if evolution_base_rendering == idx}
                                            <span class="ml-2 text-[10px] text-[hsl(var(--primary))]">Base</span>
                                        {/if}
                                        {#if rendering?.is_system_font}
                                            <span class="ml-2 text-[10px] text-[hsl(var(--muted-foreground))]">System</span>
                                        {/if}
                                    </button>
                                    {#if !rendering?.is_system_font}
                                        <div
                                            class="absolute right-1 top-1/2 flex -translate-y-1/2 gap-1 opacity-0 transition-opacity group-hover:opacity-100"
                                        >
                                            <button
                                                class="ui-button-ghost h-6 px-2 text-[10px]"
                                                disabled={evolve_disabled}
                                                onclick={(event) => {
                                                    event.stopPropagation();
                                                    render_again_from_rendering(idx);
                                                }}
                                            >
                                                Render
                                            </button>
                                            <button
                                                class="ui-button-ghost h-6 px-2 text-[10px]"
                                                disabled={evolve_disabled}
                                                onclick={(event) => {
                                                    event.stopPropagation();
                                                    evolve_from_rendering(idx);
                                                }}
                                            >
                                                Evolve
                                            </button>
                                            <button
                                                class="ui-button-ghost h-6 px-2 text-[10px]"
                                                disabled={evolve_disabled}
                                                onclick={(event) => {
                                                    event.stopPropagation();
                                                    replace_rendering_from_base(idx);
                                                }}
                                            >
                                                X
                                            </button>
                                        </div>
                                    {/if}
                                </div>
                                {#key `${idx}-${rendering.render_version ?? 0}`}
                                    <textarea
                                        class="h-24 resize-none rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--background))] p-2 linkor text-sm cursor-pointer"
                                        readonly
                                        value={rendering.text}
                                        onclick={() => choose_evolution_rendering(idx)}
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

    {#if evolution_shape_dialog_open}
        <div class="fixed inset-0 z-[56] flex items-center justify-center bg-black/50 p-4">
            <div class="ui-card w-full max-w-4xl p-4">
                <div class="text-sm font-semibold">Evolution Config Shape</div>
                <pre
                    class="mt-3 max-h-[65vh] overflow-auto whitespace-pre rounded-md border border-[hsl(var(--input))] bg-[hsl(var(--muted))] p-3 text-xs"
                >{EVOLUTION_CONFIG_SHAPE}</pre>
                <div class="mt-3 flex justify-end">
                    <button class="ui-button-primary" onclick={() => (evolution_shape_dialog_open = false)}>
                        Close
                    </button>
                </div>
            </div>
        </div>
    {/if}
{/if}
