from random import randint


def collapse(s: str) -> str:
    res = ''
    assert (s.count('$') % 2 == 0)
    while s.count('$') > 0:
        first = s.find('$')
        second = first + 1 + s[first+1:].find('$')

        assert (s[first+1:second].count('^') == 1)
        left, right = map(int, s[first+1:second].split('^'))

        s = s[:first] + str(randint(left, right)) + s[second+1:]

    return s


def process_line(s: str) -> list[str]:
    s = s.strip()
    data = s.split(' ')
    assert (len(data) == 2)

    res = []
    n, config = (int(data[0]), data[1])
    for _ in range(n):
        res.append(collapse(config))

    return res


def main():
    res = process_line('10 $10^20$,3,constant;17;$0^100$,$-10^-5$')
    res.sort(key=lambda x: int(x.split(',')[0]))
    print(*res, sep='\n')


if __name__ == '__main__':
    main()
