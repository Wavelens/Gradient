/*
 * SPDX-FileCopyrightText: 2024 Wavelens UG <info@wavelens.io>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR WL-1.0
 */

use std::net::{SocketAddr, ToSocketAddrs};

use super::consts::*;

pub fn url_to_addr(host: &str, port: i32) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let port = port as usize;

    if !PORT_RANGE.contains(&port) {
        return Err(format!(
            "port out of range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        )
        .into());
    }

    let uri = format!("{}:{}", host, port);
    let url = uri
        .to_socket_addrs()?
        .next()
        .ok_or(format!("{} is not a valid address", uri))?;
    Ok(url)
}

pub fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` is not a port number"))?;

    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}

pub fn greater_than_zero<
    T: std::str::FromStr + std::cmp::PartialOrd + std::fmt::Display + Default,
>(
    s: &str,
) -> Result<T, String> {
    let num: T = s
        .parse()
        .map_err(|_| format!("`{}` is not a valid number", s))?;

    if num > T::default() {
        Ok(num)
    } else {
        Err(format!("`{}` is not larger than 0", s))
    }
}

pub fn hex_to_vec(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("invalid hex string".to_string());
    }

    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

pub fn vec_to_hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn repository_url_to_nix(url: &str, commit_hash: &str) -> Result<String, String> {
    let url = if url.starts_with("ssh://") {
        format!("git+{}", url)
    } else {
        url.to_string()
    };

    Ok(format!("{}?rev={}", url, commit_hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use git_url_parse::normalize_url;

    #[test]
    fn test_url_to_addr() {
        let addr = url_to_addr("127.0.0.1", 8080).unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:8080");

        let addr = url_to_addr("localhost", 8080).unwrap();
        assert_eq!(addr.to_string(), "[::1]:8080");

        let addr = url_to_addr("127.0.0.1", 65536).unwrap_err();
        assert_eq!(addr.to_string(), "port out of range 1-65535");

        let addr = url_to_addr("127.0.0.1", 0).unwrap_err();
        assert_eq!(addr.to_string(), "port out of range 1-65535");

        let addr = url_to_addr("127.0.0.1", -1).unwrap_err();
        assert_eq!(addr.to_string(), "port out of range 1-65535");

        let addr = url_to_addr("::1", 8080).unwrap();
        assert_eq!(addr.to_string(), "[::1]:8080");

        let addr = url_to_addr(":::1", 8080).unwrap_err();
        assert_eq!(
            addr.to_string(),
            "failed to lookup address information: Name or service not known"
        );
    }

    #[test]
    fn test_port_in_range() {
        let port = port_in_range("8080").unwrap();
        assert_eq!(port, 8080);

        let port = port_in_range("65535").unwrap();
        assert_eq!(port, 65535);

        let port = port_in_range("65536").unwrap_err();
        assert_eq!(port, "port not in range 1-65535");

        let port = port_in_range("0").unwrap_err();
        assert_eq!(port, "port not in range 1-65535");
    }

    #[test]
    fn test_greater_than_zero() {
        let num = greater_than_zero::<u32>("1").unwrap();
        assert_eq!(num, 1);

        let num = greater_than_zero::<usize>("0").unwrap_err();
        assert_eq!(num, "`0` is not larger than 0");

        let num = greater_than_zero::<u32>("-1").unwrap_err();
        assert_eq!(num, "`-1` is not a valid number");

        let num = greater_than_zero::<i32>("-1").unwrap_err();
        assert_eq!(num, "`-1` is not larger than 0");

        let num = greater_than_zero::<u32>("a").unwrap_err();
        assert_eq!(num, "`a` is not a valid number");

        let num = greater_than_zero::<f32>("1.0").unwrap();
        assert_eq!(num, 1.0);
    }

    #[test]
    fn test_hex_to_vec() {
        let vec = hex_to_vec("68656c6c6f").unwrap();
        assert_eq!(vec, vec![0x68, 0x65, 0x6c, 0x6c, 0x6f]);

        let vec = hex_to_vec("11c2f8505c234697ccabbc96e5b8a76daf0f31d3").unwrap();
        assert_eq!(
            vec,
            vec![
                0x11, 0xc2, 0xf8, 0x50, 0x5c, 0x23, 0x46, 0x97, 0xcc, 0xab, 0xbc, 0x96, 0xe5, 0xb8,
                0xa7, 0x6d, 0xaf, 0x0f, 0x31, 0xd3
            ]
        );

        let vec = hex_to_vec("68656c6c6").unwrap_err();
        assert_eq!(vec.to_string(), "invalid hex string");

        let vec = hex_to_vec("68656c6c6g").unwrap_err();
        assert_eq!(vec.to_string(), "invalid digit found in string");
    }

    #[test]
    fn test_repository_url_to_nix() {
        let url = repository_url_to_nix(
            normalize_url("git@github.com:Wavelens/Gradient.git")
                .unwrap()
                .as_str(),
            "11c2f8505c234697ccabbc96e5b8a76daf0f31d3",
        )
        .unwrap();
        assert_eq!(url, "git+ssh://git@github.com/Wavelens/Gradient.git?rev=11c2f8505c234697ccabbc96e5b8a76daf0f31d3");

        let url = repository_url_to_nix(
            normalize_url("https://github.com/Wavelens/Gradient.git")
                .unwrap()
                .as_str(),
            "11c2f8505c234697ccabbc96e5b8a76daf0f31d3",
        )
        .unwrap();
        assert_eq!(
            url,
            "https://github.com/Wavelens/Gradient.git?rev=11c2f8505c234697ccabbc96e5b8a76daf0f31d3"
        );
    }
}