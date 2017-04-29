/*
The data structure for the CPU load history graph
*/

const GRAPH_SIZE: usize = 51;
type GraphType = f64;

pub struct Graph {
    data: [GraphType; GRAPH_SIZE]
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            data: [0.0; GRAPH_SIZE]
        }
    }

    //works like a queue
    pub fn push(&mut self, entry: GraphType) {
        for i in 0..GRAPH_SIZE - 1 {
            self.data[i] = self.data[i+1];
        }
        self.data[GRAPH_SIZE - 1] = entry;
    }

    pub fn height_values(&self, max: usize) -> [usize; GRAPH_SIZE] {
        let mut result = [0; GRAPH_SIZE];
        for i in 0..GRAPH_SIZE {
            result[i] = (self.data[i] * max as GraphType) as usize;
        }
        result
    }
}
