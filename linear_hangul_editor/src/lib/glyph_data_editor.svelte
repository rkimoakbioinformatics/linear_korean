<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    let glyph_name = $state("");
    let glyph_data = $state("");
    let prev_glyph_name = $state("");
    let { glyph_set, ...props } = $props();

    /**
     * @param {any} event
     */
    async function loadGlyphData(event) {
        if (glyph_set == "" || glyph_name == "") {
            return;
        }
        if (prev_glyph_name != "" && glyph_data != "") {
            await invoke("save_glyph_data", {
                glyphSet: glyph_set,
                glyphName: prev_glyph_name,
                glyphData: glyph_data,
            });
        }
        glyph_data = await invoke("get_glyph_data", {
            glyphSet: glyph_set,
            glyphName: glyph_name,
        });
        prev_glyph_name = glyph_name;
    }

    /**
     * @param {any} event
     */
    export async function loadGlyphDataWithoutSave(event) {
        console.log("loading glyph_set:", glyph_set);
        if (glyph_set == "" || glyph_name == "") {
            return;
        }
        glyph_data = await invoke("get_glyph_data", {
            glyphSet: glyph_set,
            glyphName: glyph_name,
        });
        prev_glyph_name = glyph_name;
    }

    /**
     * @param {any} event
     */
    export async function save(event) {
        if (glyph_name == "" || glyph_data == "" || glyph_data == "No data") {
            return;
        }
        await invoke("save_glyph_data", {
            glyphSet: glyph_set,
            glyphName: glyph_name,
            glyphData: glyph_data,
        });
    }
</script>

<div class="">
    <div>
        <select bind:value={glyph_name} onchange={loadGlyphData}>
            <option value="gieug">ㄱ</option>
            <option value="ssang_gieug">ㄲ</option>
            <option value="nieun">ㄴ</option>
            <option value="dieud">ㄷ</option>
            <option value="ssang_dieud">ㄸ</option>
            <option value="lieul">ㄹ</option>
            <option value="mieum">ㅁ</option>
            <option value="bieub">ㅂ</option>
            <option value="ssang_bieub">ㅃ</option>
            <option value="sieus">ㅅ</option>
            <option value="ssang_sieus">ㅆ</option>
            <option value="ieung">ㅇ</option>
            <option value="yesieung">iㅇ</option>
            <option value="jieuj">ㅈ</option>
            <option value="ssang_jieuj">ㅉ</option>
            <option value="chieuch">ㅊ</option>
            <option value="kieuk">ㅋ</option>
            <option value="tieut">ㅌ</option>
            <option value="pieup">ㅍ</option>
            <option value="hieuh">ㅎ</option>
            <option value="a">ㅏ</option>
            <option value="ya">ㅑ</option>
            <option value="eo">ㅓ</option>
            <option value="yeo">ㅕ</option>
            <option value="o">ㅗ</option>
            <option value="yo">ㅛ</option>
            <option value="u">ㅜ</option>
            <option value="yu">ㅠ</option>
            <option value="eu">ㅡ</option>
            <option value="i">ㅣ</option>
            <option value="ae">ㅐ</option>
            <option value="eoe">ㅔ</option>
            <option value="yeoe">ㅖ</option>
            <option value="yae">ㅒ</option>
        </select>
        <button onclick={save}>&#x1F4BE;</button>
    </div>
    <textarea class="w-full h-64 border-1 p-2" bind:value={glyph_data}
    ></textarea>
</div>

<style lang="postcss">
    @import "tailwindcss";
    button {
        @apply rounded-sm px-2 py-1 text-xs font-semibold text-white shadow-xs focus-visible:outline-2 focus-visible:outline-offset-2;
    }
</style>
