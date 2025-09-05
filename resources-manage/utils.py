def convert_cpu(cpu):
    if cpu.endswith('n'):
        return int(cpu.strip('n')) // 1000000
    elif cpu.endswith('m'):
        return int(cpu.strip('m'))
    else:
        return int(float(cpu) * 1000)

def convert_memory(mem):
    if mem.endswith('Ki'):
        return int(mem.strip('Ki')) // 1000
    elif mem.endswith('Mi'):
        return int(mem.strip('Mi'))
    elif mem.endswith('Gi'):
        return int(mem.strip('Gi')) * 1000
    else:
        return int(mem) // (1000 * 1000)

def convert_storage(st):
    if type(st) == int:
        st = str(st)
    if st.endswith("Mi"):
        st = int(st.strip("Mi"))
    elif st.endswith("Gi"):
        st = int(st.strip("Gi")) * 1000
    else:
        # given to me in B, so convert to Mi
        st = int(int(st) / 1000000)
    return st
