import sys
import io

#i = input(':')
hangeul_wanseong_unicode_start = 44032
no_char_per_chosung = 588
no_char_per_joongsung = 28
hangeul_wanseong_unicode_end = ord('힣')
chosungs = ['ㄱ', ' ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ']
joongsungs = ['ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ']
jongsungs = ['', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ']
jamo_unicode_start = 0xac00
johab_to_jamo_chosung_add = 0x1100
johab_to_jamo_joongsung_add = 0x1161
johab_to_jamo_jongsung_add = 0x11a7
jamo_joongsung_decompose_dict = {
    0x116a: [0x1169, 0x1161], # ㅘ
    0x116b: [0x1169, 0x1162], # ㅙ
    0x116c: [0x1169, 0x1175], # ㅚ
    0x116f: [0x116e, 0x1165], # ㅝ
    0x1170: [0x116e, 0x1166], # ㅞ
    0x1171: [0x116e, 0x1175], # ㅟ
    0x1174: [0x1173, 0x1175], # ㅢ
}
jamo_jongsung_decompose_dict = {
    0x11aa: [0x11a8, 0x11ba], # ㄳ
    0x11ac: [0x11ab, 0x11bd], # ㄵ
    0x11ad: [0x11ab, 0x11c2], # ㄶ
    0x11b0: [0x11af, 0x11a8], # ㄺ
    0x11b1: [0x11af, 0x11b7], # ㄻ
    0x11b2: [0x11af, 0x11b8], # ㄼ
    0x11b3: [0x11af, 0x11ba], # ㄽ
    0x11b4: [0x11af, 0x11c0], # ㄾ
    0x11b5: [0x11af, 0x11c1], # ㄿ
    0x11b6: [0x11af, 0x11c2], # ㅀ
    0x11b9: [0x11b8, 0x11ba], # ㅄ
}
fn = sys.argv[1]
wfn = fn + '_jamo.txt'
f = open(fn, encoding='utf-8')
lines = f.readlines()
f.close()
text = ''.join(lines)
wf = io.open(wfn, encoding='utf-8', mode='w')
for c in text:
    cc = ord(c)
    print(f'original={hex(cc)}')
    if cc >= hangeul_wanseong_unicode_start and cc <= hangeul_wanseong_unicode_end:
        cc = cc - hangeul_wanseong_unicode_start
        cho = int(cc / no_char_per_chosung)
        joong = int((cc - cho*no_char_per_chosung) / no_char_per_joongsung)
        jong = int(cc - cho*no_char_per_chosung - joong*no_char_per_joongsung)
        jamos = []
        johab_cho = cho + johab_to_jamo_chosung_add
        johab_joong = joong + johab_to_jamo_joongsung_add
        if jong == 0:
            johab_jong = 0
        else:
            johab_jong = jong + johab_to_jamo_jongsung_add
        print(f'decomposed={cho} {joong} {jong}')
        jamos.append(johab_cho)
        if johab_joong in jamo_joongsung_decompose_dict:
            johab_joong = jamo_joongsung_decompose_dict[johab_joong]
        else:
            johab_joong = [johab_joong]
        jamos.extend(johab_joong)
        if johab_jong in jamo_jongsung_decompose_dict:
            johab_jong = jamo_jongsung_decompose_dict[johab_jong]
        else:
            johab_jong = [johab_jong]
        jamos.extend(johab_jong)
        print(f'jamos={[hex(v) for v in jamos]}')
        jamos = [chr(jamo) for jamo in jamos if jamo != 0]
        s = ''.join(jamos)
        print(f's={s}')
        wf.write(s)
        print(''.join(jamos), end='', flush=True)
        print()
    else:
        print(c, end='', flush=True)
        wf.write(c)
print()
wf.close()
print(chr(0x1100), chr(0x1162))
