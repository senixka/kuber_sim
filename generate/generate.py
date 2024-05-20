from random import randint, uniform
from os.path import isfile, join
from os import listdir


def collapse(s: str) -> str:
    # Base check
    assert s.count("$") % 2 == 0
    # Collapse all constructions
    while s.count("$") > 0:
        # Find nearest $a^b$ construction
        first = s.find("$")
        second = first + 1 + s[first + 1 :].find("$")
        assert s[first + 1 : second].count("^") == 1

        # Should we use int or float type
        if s[first + 1 : second].count(".") > 0:
            left, right = map(float, s[first + 1 : second].split("^"))
            s = s[:first] + str(round(uniform(left, right), 3)) + s[second + 1 :]
        else:
            left, right = map(int, s[first + 1 : second].split("^"))
            s = s[:first] + str(randint(left, right)) + s[second + 1 :]

    return s


def process_line(s: str) -> list[str]:
    # Parse data from string
    data = s.split(" ")
    assert len(data) == 2
    number_of_collapses, template_string = (int(data[0]), data[1])

    # Return generated set of pods
    return [collapse(template_string) for _ in range(number_of_collapses)]


def main():
    # Get all files in input directory
    template_files = [f for f in listdir("./data_in/") if isfile(join("./data_in/", f))]
    # For each template file generate trace
    for file_name in template_files:
        # Store generated pods
        csv_res = []
        # Read generator file
        with open("./data_in/" + file_name, "r") as fin:
            # Process each line of template file
            for line in fin:
                line = line.strip()
                if line == "":
                    continue
                # Generate part of the trace from this lien
                csv_res.extend(process_line(line))
        # Sort pods by time
        csv_res.sort(key=lambda x: float(x.split(";")[0]))
        # Write trace to output file
        with open("./data_out/" + file_name, "w") as fout:
            fout.write('\n'.join(csv_res))


if __name__ == "__main__":
    main()
