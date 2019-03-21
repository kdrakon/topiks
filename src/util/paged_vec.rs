pub struct PagedVec<'a, A: 'a> {
    page_length: usize,
    vec: &'a Vec<A>,
}

impl<'a, A> PagedVec<'a, A> {
    pub fn from(vec: &'a Vec<A>, page_length: usize) -> PagedVec<'a, A> {
        PagedVec { page_length, vec }
    }

    pub fn page(&'a self, index: usize) -> Option<(usize, Vec<&'a A>)> {
        let mut paged = self.vec.chunks(self.page_length);
        let opt_page = paged.nth((index as f32 / self.page_length as f32).floor() as usize);

        opt_page.map(|page| (index % self.page_length, page.iter().collect::<Vec<&'a A>>()))
    }
}
