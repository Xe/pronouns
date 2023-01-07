import csv

files = []

with open("./pronouns.tab") as fin:
    rdr = csv.reader(fin, delimiter="\t")
    for row in rdr:
        nom, acc, gen, pos, ref = row

        fname = f"pronouns/{nom}-{acc}-{gen}-{pos}-{ref}.dhall"
        fname = fname.replace("'", "_")

        with open(fname, "w") as fout:
            fout.write(f"""
let PronounSet = ../types/PronounSet.dhall

in PronounSet::{{
    , nominative = "{nom}"
    , accusative = "{acc}"
    , determiner = "{gen}"
    , possessive = "{pos}"
    , reflexive = "{ref}"
    , singular = True
}}
            """)

        files.append(fname)

print(files)

with open("./package.dhall", "w") as fout:
    fout.write("[\n")

    for fname in files:
        fout.write(f", ./{fname}\n")

    fout.write("]")
