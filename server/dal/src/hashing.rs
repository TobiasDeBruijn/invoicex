use sha2::Digest;

/// Generate a hash for the provided input.
/// The returned String is a hash of the input and it's salt
///
/// # Errors
///
/// If hashing fails
pub fn hash(input: &str, salt: [u8; 16], pepper: &str) -> Result<String, bcrypt::BcryptError> {
    let mut hasher = sha2::Sha512_256::new();

    hasher.update(input);
    hasher.update(pepper);

    let hash = base64::encode(hasher.finalize());
    let bcrypt = bcrypt::hash_with_salt(&hash, 10, salt)?.format_for_version(bcrypt::Version::TwoB);

    Ok(bcrypt)
}

/// Verify an input is the same as the stored hash. The same `pepper` must be used
///
/// # Errors
///
/// If verifying fails
pub fn verify(stored_hash: &str, input: &str, pepper: &str) -> Result<bool, bcrypt::BcryptError> {
    let mut hasher = sha2::Sha512_256::new();

    hasher.update(input);
    hasher.update(pepper);

    let hash = base64::encode(hasher.finalize());
    let correct = bcrypt::verify(hash, stored_hash)?;

    Ok(correct)
}

#[cfg(test)]

mod test {
    use super::{hash, verify};

    fn salty(salt: &str) -> [u8; 16] {
        let bytes = salt.as_bytes();
        if bytes.len() != 16 {
            panic!("Invalid salt length");
        }

        let mut result = [0u8; 16];
        for idx in 0..bytes.len() {
            result[idx] = bytes[idx];
        }

        result
    }

    #[test]
    fn long_password() {
        let password = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz123456789!@#$%^&*()_+=-.,/<>?ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz123456789!@#$%^&*()_+=-.,/<>?ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz123456789!@#$%^&*()_+=-.,/<>?";

        assert!(hash(password, salty("013456789ABCDEF"), "Baz").is_ok());
    }

    #[test]
    fn utf8() {
        let password = r#"1ǯ;ؔ洲񏒡lٯ>睵帠t0򴢷򞈚𠁺ꪲ2ɧޞ顬أz형Ξ򢏟򇶔Cǣ>𾊅ܱ
            ƒd󚯋뭔̈́򾠄Ú򈭦廃i墼񺭺뼣犔G:絫ֵ藦၏O񃾴0̆Ɲ~򳯠μ񏅲¡
            򀛧񼸎҅󇛚▍ӥ஌}謹쪊񆷚荩簝ڋ򄔵~Ê΂湰ǈ򏯃󔬝ȑ)Ͻe0Bݻ󺦠ꯧG
            [񯹣٪즯P񄕤𗚕Ę;狥#񆝤뿇+kلެ󿧾ðzj2哳nܝ얂񐰅ևܐߍԞ菟
            ߺáJ՞ϴQѲ䆑䚁夸ʘQ꒾㖊򵤅𫨻騌Ӽ񱢻տ�Թ!֦j"󏮢봸򰍲Ґ􁙟"#;

        assert!(hash(password, salty("0123456789ABCDEF"), "Baz").is_ok());

        let password = r#"꩘>õ阭ܪ򤙩ԁʢΝO
            񗎕ē𼖝򜬳󀤬񎘰{򗲒؃@
            ?1朼갗ʹ򻖜㈌􌟴暻F
            互O嗎񧠙挐E򨃬<񨋥г
            󧬧ꎃٛՔ6䲋ړP䃩
            ńʏ񖸋Ɗ񛏨󎄅<�f
            K󧾴劉ڏ󲟶􍐭􉚐񭶙񾠱-
            ť֚ۧӟ4񚚐�򐺔򞧎Ӿ
            󚁝n撁鞌BΒh񸝕ｒݢ
            򘴱ⰲ򨅂󮸠񧎭tᖥ󎸟뛜"#;

        assert!(hash(password, salty("0123456789ABCDEF"), "Baz").is_ok());
    }

    #[test]
    fn correct() {
        let salt = "0123456789ABCDEF";
        let password = "XYZ";
        let pepper = "123";

        let hash = hash(password, salty(salt), pepper).unwrap();
        assert!(verify(&hash, &password, pepper).unwrap());
    }

    #[test]
    fn incorrect() {
        let salt = "0123456789ABCDEF";
        let password = "XYZ";
        let password2 = "ABC";
        let pepper = "123";

        let hash = hash(password, salty(salt), pepper).unwrap();
        assert!(!verify(&hash, &password2, pepper).unwrap());
    }
}