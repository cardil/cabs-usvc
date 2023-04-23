use actix_web::{
    error,
    http::header::{
        self,
        HeaderValue,
    },
    HttpRequest,
    HttpResponse,
    Result
};

#[derive(Debug, Clone, Copy)]
pub struct Page {
    pub num: u32,
    pub per: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub page: Page,
    pub total: isize,
}

impl Page {
    pub fn start(&self) -> isize {
        let mut p = self.num as isize;
        if p < 1 {
            p = 1;
        }
        (p - 1) * self.per as isize
    }

    pub fn stop(&self) -> isize {
        self.start() + self.per as isize - 1
    }

    pub fn to_pagination(&self, total: isize) -> Pagination {
        let mut page = self.clone();
        if self.stop() > total {
            page.num = (total / self.per as isize) as u32 + 1;
        }
        Pagination { page, total }
    }

    fn try_from_header(h: &HeaderValue) -> Result<Self> {
        let range = h.to_str()
            .map_err(error::ErrorBadRequest)?;
        parse_range(range)
    }
}

impl Default for Page {
    fn default() -> Self {
        Self { num: 1, per: 20 }
    }
}

impl TryFrom<&HttpRequest> for Page {
    fn try_from(req: &HttpRequest) -> Result<Self> {
        let maybe_header = req
            .headers()
            .get(header::RANGE);

        match maybe_header {
            None => Ok(Page::default()),
            Some(h) => Page::try_from_header(h),
        }
    }

    type Error = error::Error;
}

impl TryFrom<&HttpResponse> for Pagination {
    fn try_from(res: &HttpResponse) -> Result<Self> {
        let maybe_header = res
            .headers()
            .get(header::CONTENT_RANGE);

        match maybe_header {
            None => Err(error::ErrorBadRequest("No content range")),
            Some(h) => {
                let header_value = h.to_str()
                    .map_err(error::ErrorBadRequest)?;
                parse_pagination(header_value)
            },
        }
    }

    type Error = error::Error;
}

fn parse_range(range: &str) -> Result<Page> {
    let mut parts = range.splitn(2, '=');
    let ty = parts.next();
    if ty.is_none() {
        return Err(error::ErrorBadRequest("Invalid range"));
    }
    let ty = ty.unwrap();
    if ty != "page" {
        return Err(error::ErrorBadRequest(format!(
            "Invalid range type: {}",
            ty
        )));
    }

    let mut parts = parts
        .next()
        .ok_or(error::ErrorBadRequest("Invalid range"))?
        .split(',');

    let first = parts
        .next()
        .ok_or(error::ErrorBadRequest("Invalid range"))?;
    match parts.next() {
        Some(spec) => {
            return Err(error::ErrorBadRequest(format!(
                "Extra invalid range: {}",
                spec
            )))
        }
        None => (),
    }

    let mut parts = first.splitn(2, '-');
    let mut p = Page::default();
    p.num = parts
        .next()
        .unwrap_or(p.num.to_string().as_str())
        .parse::<u32>()
        .map_err(error::ErrorBadRequest)?;

    p.per = parts
        .next()
        .unwrap_or(p.per.to_string().as_str())
        .parse::<u16>()
        .map_err(error::ErrorBadRequest)?;
    Ok(p)
}

fn parse_pagination(header_value: &str) -> Result<Pagination> {
    let mut parts = header_value.splitn(2, ' ');
    let ty = parts.next();
    if ty.is_none() {
        return Err(error::ErrorBadRequest("Invalid range"));
    }
    let ty = ty.unwrap();
    if ty != "page" {
        return Err(error::ErrorBadRequest(format!(
            "Invalid range type: {}",
            ty
        )));
    }

    let mut parts = parts
        .next()
        .ok_or(error::ErrorBadRequest("Invalid range"))?
        .split('/');
    let mut page = Page::default();
    let mut range = parts
        .next()
        .ok_or(error::ErrorBadRequest("Invalid range"))?
        .split('-');
    page.num = range
        .next()
        .unwrap_or(page.num.to_string().as_str())
        .parse::<u32>()
        .map_err(error::ErrorBadRequest)?;

    page.per = range
        .next()
        .unwrap_or(page.per.to_string().as_str())
        .parse::<u16>()
        .map_err(error::ErrorBadRequest)?;

    let total = parts
        .next()
        .ok_or(error::ErrorBadRequest("Invalid range"))?
        .parse::<isize>()
        .map_err(error::ErrorBadRequest)?;

    Ok(Pagination { page, total })
}

impl Pagination {
    pub fn onto_response<T>(&self, res: &mut HttpResponse<T>) -> Result<()> {
        res.headers_mut().insert(
            header::ACCEPT_RANGES,
            header::HeaderValue::from_static("page"),
        );
        res.headers_mut().insert(
            header::CONTENT_RANGE,
            header::HeaderValue::from_str(&format!(
                "page {}-{}/{}",
                self.page.num, self.page.per, self.total
            ))?,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn test_parse_range() -> Result<()> {
        let req = TestRequest::default().to_http_request();
        let p = Page::try_from(&req)?;
        assert_eq!(p.num, 1);
        assert_eq!(p.per, 20);

        let req = TestRequest::default()
            .append_header((header::RANGE, "page=2"))
            .to_http_request();
        let p = Page::try_from(&req)?;
        assert_eq!(p.num, 2);
        assert_eq!(p.per, 20);

        let req = TestRequest::default()
            .append_header((header::RANGE, "page=3-50"))
            .to_http_request();
        let p = Page::try_from(&req)?;
        assert_eq!(p.num, 3);
        assert_eq!(p.per, 50);

        let req = TestRequest::default()
            .append_header((header::RANGE, "2"))
            .to_http_request();
        let res = Page::try_from(&req);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid range type: 2");

        let req = TestRequest::default()
            .append_header((header::RANGE, "bytes=2-10,33-"))
            .to_http_request();
        let res = Page::try_from(&req);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid range type: bytes");

        let req = TestRequest::default()
            .append_header((header::RANGE, "page=2-10,33-"))
            .to_http_request();
        let res = Page::try_from(&req);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Extra invalid range: 33-");

        Ok(())
    }

    #[test]
    fn test_pagination() {
        let p = Page::default();
        assert_eq!(p.start(), 0);
        assert_eq!(p.stop(), 19);

        let p = Page { num: 1, per: 2 };
        assert_eq!(p.start(), 0);
        assert_eq!(p.stop(), 1);

        let p = Page { num: 2, per: 10 };
        assert_eq!(p.start(), 10);
        assert_eq!(p.stop(), 19);

        let mut p = Page::default();
        p.num = 2;

        assert_eq!(p.start(), 20);
        assert_eq!(p.stop(), 39);
    }
}
