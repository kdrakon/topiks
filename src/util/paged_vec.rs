pub struct PagedVec<'a, A: 'a> {
    indexes: usize,
    page_length: usize,
    pages: Vec<Vec<&'a A>>,
}

impl<'a, A> PagedVec<'a, A> {
    pub fn from(vec: &'a Vec<A>, page_length: usize) -> PagedVec<'a, A> {
        PagedVec {
            indexes: vec.len(),
            page_length,
            pages: vec.chunks(page_length).map(|slice| slice.iter().collect::<Vec<&'a A>>()).collect::<Vec<Vec<&'a A>>>(),
        }
    }

    pub fn page(&'a self, index: usize) -> Option<(usize, &'a Vec<&'a A>)> {
        self.pages.get((index as f32 / self.page_length as f32).floor() as usize).map(|page| (index % self.page_length, page))
    }
}
