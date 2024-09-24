#[allow(dead_code)]
pub struct Stack<T> {
    tail: usize,
    data: Vec<Option<T>>,
}

#[allow(dead_code)]
impl<T> Stack<T> {
    pub fn new(size: usize) -> Self {
        Stack {
            tail: 0,
            data: Vec::with_capacity(size),
        }
    }

    pub fn push(&mut self, element: T) {
        self.push_element(element);

        if self.tail + 1 < self.data.capacity() {
            self.tail += 1;
        } else {
            self.tail = 0;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let prev = match self.tail {
            0 => 0,
            _ => {
                self.tail -= 1;
                self.tail
            }
        };

        self.data[prev].take()
    }

    fn push_element(&mut self, element: T) {
        if self.is_full() {
            self.data.push(Some(element)); // grow the vec by pushing an an element
        } else {
            // we need to clean-up memory of the previous value by extracting it in the scope
            self.data[self.tail].take();
            self.data[self.tail] = Some(element);
        }
    }

    fn is_full(&self) -> bool {
        self.tail == self.data.len() && self.tail < self.data.capacity()
    }
}