import os
from math import log2

INTS = ["u8", "u16", "u32", "u64"]

def filter_structs(raw):
    structs = []
    in_struct = False
    for l in raw:
        if l[:6] == "struct":
            in_struct = True
        if in_struct:
            structs.append(l)
        if l[:2] == "};":
            in_struct = False
    return structs

def translate_structs(structs):
    parsed = []
    in_block_comment = False
    for l in structs:
        if l[:2] == "};":
            parsed.append("}\n\n")
            continue
        if l[0] == "{":
            continue
        if l[:6] == "struct":
            tokens = l.split()
            _, structname = tokens[:2]
            comment = " ".join(tokens[2:])
            if comment[0] == "{":
                comment = comment[1:]
            if comment:
                comment = comment.replace("/*", "////")
                comment = comment.replace("*/", "")
                comment += "\n"
            structname = "".join(t.capitalize() for t in structname.split("_"))
            parsed.append(f"{comment}pub struct {structname} {{\n")
            continue
        if len(l) < 5:
            parsed.append(l)
            continue
        if l[4] in "s }":
            parsed.append(l)
            continue
        tokens = l.split()
        if tokens[0] == "/*":
            in_block_comment = True
        if tokens[0] == "*/":
            in_block_comment = False
        if not in_block_comment and tokens[0] != "*/":
            typestr, name = tokens[:2]
            comment = " ".join(tokens[2:])
            if typestr[:2] == "__":
                typestr = typestr[2:]
            elif typestr == "char":
                typestr = "u8"
            name = name[:-1]
            if "[" in name:
                name, arrlen = name.split("[")
                arrlen = arrlen[:-1]
            else:
                arrlen = ""
            if comment:
                comment = comment.replace("/*", "////")
                comment = comment.replace("*/", "")
                comment = f"    {comment}\n"
            if arrlen:
                pl = f"{comment}    {name}: [{typestr}; {arrlen}],\n"
            else:
                pl = f"{comment}    {name}: {typestr},\n"
            parsed.append(pl)
        else:
            l = l.replace("/*", "////")
            l = l.replace(" *", "////")
            parsed.append(l)
    return parsed

def translate_consts(raw):
    parsed = []
    types = dict()
    for l in raw:
        result = ""
        if l[:3] in ["/* ", " * ", " */", "/*\n"]:
            result = f"//// {l[3:].strip()}\n"
            parsed.append(result)
            continue
        if l[:7] == "#define":
            tokens = l.split()
            name = tokens[1]
            try:
                token_idx = tokens.index("/*")
                comment = " ".join(tokens[token_idx:])
                comment = comment.replace("/*", "////")
                comment = comment.replace("*/", "")
                comment += "\n"
            except ValueError:
                token_idx = 0
                comment = ""
            if token_idx:
                value = " ".join(tokens[2:token_idx])
            else:
                value = " ".join(tokens[2:])
            if value[0] == '"':
                typestr = "&str"
            elif value.isnumeric() or value[:2] == "0x":
                if value.isnumeric():
                    n = int(value)
                    bytesize = 1
                else:
                    n = int(value, 16)
                    bytesize = (len(value) - 2) / 2
                idx = max(int(log2(bytesize)), 0)
                for j, tn in enumerate(INTS[idx:]):
                    k = 1 << (j + idx)
                    if n < (1 << (8 * k)):
                        typestr = tn
                        break
            else:
                for token in value.split():
                    typestr = types.get(token.replace(";", "").replace("(", "").replace(")", ""))
                    if typestr:
                        break
                if not typestr:
                    typestr = "TBD"
            types[name] = typestr
            result = f"{comment}const {name}: {typestr} = {value};\n"
            parsed.append(result)
    return parsed
            

if __name__ == "__main__":
    os.chdir("/home/christian/rust/titanium/kernel/src/filesystem/")
    with open("consts.h") as f:
        raw = f.readlines()
    parsed = translate_consts(raw)
    with open("structs_pre.rs", "w") as f:
        f.writelines(parsed)