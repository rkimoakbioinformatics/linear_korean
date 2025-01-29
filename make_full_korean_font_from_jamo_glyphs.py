# pip install fonttools
# pip install brotli
from fontTools.pens.ttGlyphPen import TTGlyphPen
from typing import Optional

cho_conversion_table = {
    0x1113: [0x1102, 0x1100],
    0x1114: [0x1102, 0x1102],
    0x1115: [0x1102, 0x1103],
    0x1116: [0x1102, 0x1107],
    0x1117: [0x1103, 0x1100],
    0x1118: [0x1105, 0x1102],
    0x1119: [0x1105, 0x1105],
    0x111a: [0x1105, 0x1112],
}

jung_conversion_table = {
    0x116a: [0x1169, 0x1161],
    0x116b: [0x1169, 0x1162],
    0x116c: [0x1169, 0x1175],
    0x116f: [0x116e, 0x1165],
    0x1170: [0x116e, 0x1166],
    0x1171: [0x116e, 0x1175],
    0x1174: [0x1173, 0x1175],
}

jong_conversion_table = {
    0x11aa: [0x11a8, 0x11ba],
    0x11ac: [0x11ab, 0x11bd],
    0x11ad: [0x11ab, 0x11c2],
    0x11b0: [0x11af, 0x11a8],
    0x11b1: [0x11af, 0x11b7],
    0x11b2: [0x11af, 0x11b8],
    0x11b3: [0x11af, 0x11ba],
    0x11b4: [0x11af, 0x11c0],
    0x11b5: [0x11af, 0x11c1],
    0x11b6: [0x11af, 0x11c2],
    0x11b9: [0x11b8, 0x11ba],
    0x11a7: [],
}

def get_cho_code_nos(code_no) -> Optional[list[int]]:
    if code_no in cho_conversion_table:
        return cho_conversion_table[code_no]
    else:
        return None

def get_jung_code_nos(code_no) -> Optional[list[int]]:
    if code_no in jung_conversion_table:
        return jung_conversion_table[code_no]
    else:
        return None

def get_jong_code_nos(code_no) -> Optional[list[int]]:
    if code_no in jong_conversion_table:
        return jong_conversion_table[code_no]
    else:
        return None

def add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, code_no: int, glyph_list_list: list[list[int]]):
    _ = vmtx
    pen = TTGlyphPen(glyph_set) # type: ignore
    font_width = 0
    glyph_gap = 50
    char_gap = 200
    for i, code_no_c in enumerate(glyph_list_list[0]):
        glyph_name = cmap[code_no_c]
        if i == 0:
            gap = char_gap
        else:
            gap = glyph_gap
        pen.addComponent(glyph_name, (1, 0, 0, 1, font_width + gap, 0))
        font_width += hmtx[glyph_name][0] + gap
    no_jong = len(glyph_list_list[2]) == 0
    last_jung_idx = len(glyph_list_list[1]) - 1
    for i, code_no_c in enumerate(glyph_list_list[1]):
        glyph_name = cmap[code_no_c]
        #pen.addComponent(glyph_name, (1, 0, 0, 1, cho_jung_width, -jung_height*3/4))
        pen.addComponent(glyph_name, (1, 0, 0, 1, font_width + glyph_gap, 0))
        if i == last_jung_idx and no_jong:
            font_width += hmtx[glyph_name][0] + glyph_gap
        else:
            font_width += hmtx[glyph_name][0] + glyph_gap
    if no_jong:
        font_width += char_gap
    else:
        last_jong_idx = len(glyph_list_list[2]) - 1
        for i, code_no_c in enumerate(glyph_list_list[2]):
            glyph_name = cmap[code_no_c]
            #pen.addComponent(glyph_name, (1, 0, 0, 1, jong_width, -jong_height*3/4))
            pen.addComponent(glyph_name, (1, 0, 0, 1, font_width + glyph_gap, 0))
            if i == last_jong_idx:
                font_width += hmtx[glyph_name][0] + char_gap
            else:
                font_width += hmtx[glyph_name][0] + glyph_gap
    glyph = pen.glyph()
    glyph_name = f"uni{code_no:04x}"
    glyf[glyph_name] = glyph # type: ignore
    cmap[code_no] = glyph_name
    font_height = 1600
    #print(f"{code_no:04x} {glyph_name} {char_width} {char_height}")
    hmtx[glyph_name] = (font_width, font_height) # type: ignore
    maxp.numGlyphs += 1 # type: ignore

def map_chosungs(glyph_set, glyf, cmap, hmtx, vmtx, maxp):
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3131, [[0x1100], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3132, [[0x1101], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3133, [[0x1100, 0x1109], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3134, [[0x1102], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3135, [[0x1102, 0x110c], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3136, [[0x1102, 0x1112], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3137, [[0x1103], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3138, [[0x1104], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3139, [[0x1105], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x313a, [[0x1105, 0x1100], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x313b, [[0x1105, 0x1106], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x313c, [[0x1105, 0x1107], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x313d, [[0x1105, 0x1109], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x313e, [[0x1105, 0x1110], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x313f, [[0x1105, 0x1111], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3140, [[0x1105, 0x1112], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3141, [[0x1106], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3142, [[0x1107], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3143, [[0x1108], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3144, [[0x1107, 0x1109], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3145, [[0x1109], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3146, [[0x110a], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3147, [[0x110b], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3148, [[0x110c], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3149, [[0x110d], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x314a, [[0x110e], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x314b, [[0x110f], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x314c, [[0x1110], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x314d, [[0x1111], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x314e, [[0x1112], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x314f, [[0x1161], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3150, [[0x1162], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3151, [[0x1163], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3152, [[0x1164], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3153, [[0x1165], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3154, [[0x1166], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3155, [[0x1167], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3156, [[0x1168], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3157, [[0x1169], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3158, [[0x1169, 0x1161], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3159, [[0x1169, 0x1162], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x315a, [[0x1169, 0x1175], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x315b, [[0x116d], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x315c, [[0x116e], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x315d, [[0x116e, 0x1165], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x315e, [[0x116e, 0x1166], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x315f, [[0x116e, 0x1175], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3160, [[0x1172], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3161, [[0x1173], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3162, [[0x1173, 0x1175], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3163, [[0x1175], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3165, [[0x1102, 0x1102], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x3166, [[0x1102, 0x1103], [], []])
    add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, 0x318d, [[0x119e], [], []])

if __name__ == "__main__":
    import sys
    from fontTools.ttLib import TTFont
    ttf_path = sys.argv[1]
    font_name = ".".join(ttf_path.split(".")[0:-1])
    new_font_name = font_name + ".complete"
    font = TTFont(ttf_path)
    cmap = font.getBestCmap()
    glyf = font["glyf"]
    hmtx = font["hmtx"]
    vmtx = None
    maxp = font["maxp"]
    glyph_set = font.getGlyphSet()
    glyph_names = font.getGlyphOrder()
    map_chosungs(glyph_set, glyf, cmap, hmtx, vmtx, maxp)
    cho_start = 0x1100
    jung_start = 0x1161
    jong_start = 0x11a7
    code_no_start = 0xac00
    code_no_end = 0xd7a3
    for code_no in range(code_no_start, code_no_end + 1):
        code_offset = code_no - code_no_start
        jong_offset = code_offset % 28
        jung_offset = (code_offset // 28) % 21
        cho_offset = (code_offset // 28) // 21
        cho_code_no = cho_start + cho_offset
        jung_code_no = jung_start + jung_offset
        jong_code_no = jong_start + jong_offset
        if cho_code_no in cmap:
            cho_code_nos = [cho_code_no]
        else:
            cho_code_nos = get_cho_code_nos(cho_code_no)
        if jung_code_no in cmap:
            jung_code_nos = [jung_code_no]
        else:
            jung_code_nos = get_jung_code_nos(jung_code_no)
        if jong_code_no in cmap:
            jong_code_nos = [jong_code_no]
        else:
            jong_code_nos = get_jong_code_nos(jong_code_no)
        if cho_code_nos is None or jung_code_nos is None or jong_code_nos is None:
            continue
        add_glyph(glyph_set, glyf, cmap, hmtx, vmtx, maxp, code_no, [cho_code_nos, jung_code_nos, jong_code_nos])
    font.save(f"{new_font_name}.ttf")
    font.flavor = "woff"
    font.save(f"{new_font_name}.woff")
    font.flavor = "woff2"
    font.save(f"{new_font_name}.woff2")
