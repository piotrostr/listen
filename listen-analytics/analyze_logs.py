import re
from collections import Counter

# Pattern matches "kit[" followed by any digits "]:"
split_pattern = r"kit\[\d+\]:"

if __name__ == "__main__":
    with open("logs.txt", "r") as file:
        lines = file.readlines()
        # Extract everything after kit[number]: pattern
        log_messages = []
        for line in lines:
            match = re.split(split_pattern, line.strip())
            if len(match) > 1:
                log_messages.append(match[1].strip())

    counts = Counter(log_messages)

    # Print each value and its count
    for value, count in counts.most_common():
        print(f"{count:>5} | {value}")
