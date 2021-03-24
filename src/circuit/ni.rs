use super::es_graph::CompGraphWork;
use super::sl_sc::SlSharedCircuit;

/// Set of input shares required to simulate a set of probes.
pub fn sim_set<T: IntoIterator<Item = u32>>(circ: &SlSharedCircuit, probes: T) -> Vec<u32> {
    let probes = probes
        .into_iter()
        .map(|x| x as usize)
        .collect::<Vec<usize>>();
    let (mut c, _) = CompGraphWork::from_circ_probes(circ, probes.iter().copied());
    c.simplify();
    let inputs = c.inputs();
    return inputs;
}
