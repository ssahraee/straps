def IronMasktoStraps(lines, d):
    #Take input a script implementable in IronMask and convert it to a Straps script
    #Assumes operations are either + or * or =
    randoms = []
    c, (x, y), z = op_preamble(d, 2)
    for line in lines:
        if line.upper().startswith("#RANDOMS"):
            randoms = line.split()[1:]
            [c.var(str(rand), kind="random") for rand in randoms]
        else:
            #We split the line into chunks. Each line will have length 5 (for + or *) or 3 (for reassignment =)
            tokens = line.strip().split()
            if tokens:
                #Create a list of intermediate variables, allowing repeated assignments
                dest = tokens[0]
                c.var(str(dest))
                if len(tokens)==5:
                    if tokens[3] == "+":
                        c.l_sum(dest, (tokens[2], tokens[4]))
                    elif tokens[3] == "*":
                        c.l_prod(dest, (tokens[2], tokens[4]))
                    elif len(tokens)==3: #"="
                        c.assign(dest, tokens[2])
    return c
