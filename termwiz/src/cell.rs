//! Model a cell in the terminal display
use crate::color::{ColorAttribute, PaletteIndex};
pub use crate::emoji::Presentation;
pub use crate::escape::osc::Hyperlink;
use crate::image::ImageCell;
use crate::widechar_width::WcWidth;
#[cfg(feature = "use_serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::mem;
use std::sync::Arc;

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SmallColor {
    Default,
    PaletteIndex(PaletteIndex),
}

impl Default for SmallColor {
    fn default() -> Self {
        Self::Default
    }
}

impl Into<ColorAttribute> for SmallColor {
    fn into(self) -> ColorAttribute {
        match self {
            Self::Default => ColorAttribute::Default,
            Self::PaletteIndex(idx) => ColorAttribute::PaletteIndex(idx),
        }
    }
}

/// Holds the attributes for a cell.
/// Most style attributes are stored internally as part of a bitfield
/// to reduce per-cell overhead.
/// The setter methods return a mutable self reference so that they can
/// be chained together.
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq)]
pub struct CellAttributes {
    attributes: u16,
    /// The foreground color
    foreground: SmallColor,
    /// The background color
    background: SmallColor,
    /// Relatively rarely used attributes spill over to a heap
    /// allocated struct in order to keep CellAttributes
    /// smaller in the common case.
    fat: Option<Box<FatAttributes>>,
}

impl std::fmt::Debug for CellAttributes {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("CellAttributes")
            .field("attributes", &self.attributes)
            .field("intensity", &self.intensity())
            .field("underline", &self.underline())
            .field("blink", &self.blink())
            .field("italic", &self.italic())
            .field("reverse", &self.reverse())
            .field("strikethrough", &self.strikethrough())
            .field("invisible", &self.invisible())
            .field("wrapped", &self.wrapped())
            .field("overline", &self.overline())
            .field("semantic_type", &self.semantic_type())
            .field("foreground", &self.foreground)
            .field("background", &self.background)
            .field("fat", &self.fat)
            .finish()
    }
}

#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct FatAttributes {
    /// The hyperlink content, if any
    hyperlink: Option<Arc<Hyperlink>>,
    /// The image data, if any
    image: Vec<Box<ImageCell>>,
    /// The color of the underline.  If None, then
    /// the foreground color is to be used
    underline_color: ColorAttribute,
    foreground: ColorAttribute,
    background: ColorAttribute,
}

/// Define getter and setter for the attributes bitfield.
/// The first form is for a simple boolean value stored in
/// a single bit.  The $bitnum parameter specifies which bit.
/// The second form is for an integer value that occupies a range
/// of bits.  The $bitmask and $bitshift parameters define how
/// to transform from the stored bit value to the consumable
/// value.
macro_rules! bitfield {
    ($getter:ident, $setter:ident, $bitnum:expr) => {
        #[inline]
        pub fn $getter(&self) -> bool {
            (self.attributes & (1 << $bitnum)) == (1 << $bitnum)
        }

        #[inline]
        pub fn $setter(&mut self, value: bool) -> &mut Self {
            let attr_value = if value { 1 << $bitnum } else { 0 };
            self.attributes = (self.attributes & !(1 << $bitnum)) | attr_value;
            self
        }
    };

    ($getter:ident, $setter:ident, $bitmask:expr, $bitshift:expr) => {
        #[inline]
        pub fn $getter(&self) -> u16 {
            (self.attributes >> $bitshift) & $bitmask
        }

        #[inline]
        pub fn $setter(&mut self, value: u16) -> &mut Self {
            let clear = !($bitmask << $bitshift);
            let attr_value = (value & $bitmask) << $bitshift;
            self.attributes = (self.attributes & clear) | attr_value;
            self
        }
    };

    ($getter:ident, $setter:ident, $enum:ident, $bitmask:expr, $bitshift:expr) => {
        #[inline]
        pub fn $getter(&self) -> $enum {
            unsafe { mem::transmute(((self.attributes >> $bitshift) & $bitmask) as u16) }
        }

        #[inline]
        pub fn $setter(&mut self, value: $enum) -> &mut Self {
            let value = value as u16;
            let clear = !($bitmask << $bitshift);
            let attr_value = (value & $bitmask) << $bitshift;
            self.attributes = (self.attributes & clear) | attr_value;
            self
        }
    };
}

/// Describes the semantic "type" of the cell.
/// This categorizes cells into Output (from the actions the user is
/// taking; this is the default if left unspecified),
/// Input (that the user typed) and Prompt (effectively, "chrome" provided
/// by the shell or application that the user is interacting with.
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum SemanticType {
    Output = 0,
    Input = 1,
    Prompt = 2,
}

impl Default for SemanticType {
    fn default() -> Self {
        Self::Output
    }
}

/// The `Intensity` of a cell describes its boldness.  Most terminals
/// implement `Intensity::Bold` by either using a bold font or by simply
/// using an alternative color.  Some terminals implement `Intensity::Half`
/// as a dimmer color variant.
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Intensity {
    Normal = 0,
    Bold = 1,
    Half = 2,
}

impl Default for Intensity {
    fn default() -> Self {
        Self::Normal
    }
}

/// Specify just how underlined you want your `Cell` to be
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Underline {
    /// The cell is not underlined
    None = 0,
    /// The cell is underlined with a single line
    Single = 1,
    /// The cell is underlined with two lines
    Double = 2,
    /// Curly underline
    Curly = 3,
    /// Dotted underline
    Dotted = 4,
    /// Dashed underline
    Dashed = 5,
}

impl Default for Underline {
    fn default() -> Self {
        Self::None
    }
}

/// Allow converting to boolean; true means some kind of
/// underline, false means none.  This is used in some
/// generic code to determine whether to enable underline.
impl Into<bool> for Underline {
    fn into(self) -> bool {
        self != Underline::None
    }
}

/// Specify whether you want to slowly or rapidly annoy your users
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Blink {
    None = 0,
    Slow = 1,
    Rapid = 2,
}

/// Allow converting to boolean; true means some kind of
/// blink, false means none.  This is used in some
/// generic code to determine whether to enable blink.
impl Into<bool> for Blink {
    fn into(self) -> bool {
        self != Blink::None
    }
}

impl Default for CellAttributes {
    fn default() -> Self {
        Self::blank()
    }
}

impl CellAttributes {
    bitfield!(intensity, set_intensity, Intensity, 0b11, 0);
    bitfield!(underline, set_underline, Underline, 0b111, 2);
    bitfield!(blink, set_blink, Blink, 0b11, 5);
    bitfield!(italic, set_italic, 7);
    bitfield!(reverse, set_reverse, 8);
    bitfield!(strikethrough, set_strikethrough, 9);
    bitfield!(invisible, set_invisible, 10);
    bitfield!(wrapped, set_wrapped, 11);
    bitfield!(overline, set_overline, 12);
    bitfield!(semantic_type, set_semantic_type, SemanticType, 0b11, 13);

    pub const fn blank() -> Self {
        Self {
            attributes: 0,
            foreground: SmallColor::Default,
            background: SmallColor::Default,
            fat: None,
        }
    }

    /// Returns true if the attribute bits in both objects are equal.
    /// This can be used to cheaply test whether the styles of the two
    /// cells are the same, and is used by some `Renderer` implementations.
    pub fn attribute_bits_equal(&self, other: &Self) -> bool {
        self.attributes == other.attributes
    }

    /// Set the foreground color for the cell to that specified
    pub fn set_foreground<C: Into<ColorAttribute>>(&mut self, foreground: C) -> &mut Self {
        let foreground: ColorAttribute = foreground.into();
        match foreground {
            ColorAttribute::Default => {
                self.foreground = SmallColor::Default;
                if let Some(fat) = self.fat.as_mut() {
                    fat.foreground = ColorAttribute::Default;
                }
                self.deallocate_fat_attributes_if_none();
            }
            ColorAttribute::PaletteIndex(idx) => {
                self.foreground = SmallColor::PaletteIndex(idx);
                if let Some(fat) = self.fat.as_mut() {
                    fat.foreground = ColorAttribute::Default;
                }
                self.deallocate_fat_attributes_if_none();
            }
            foreground => {
                self.foreground = SmallColor::Default;
                self.allocate_fat_attributes();
                self.fat.as_mut().unwrap().foreground = foreground;
            }
        }

        self
    }

    pub fn foreground(&self) -> ColorAttribute {
        if let Some(fat) = self.fat.as_ref() {
            if fat.foreground != ColorAttribute::Default {
                return fat.foreground;
            }
        }
        self.foreground.into()
    }

    pub fn set_background<C: Into<ColorAttribute>>(&mut self, background: C) -> &mut Self {
        let background: ColorAttribute = background.into();
        match background {
            ColorAttribute::Default => {
                self.background = SmallColor::Default;
                if let Some(fat) = self.fat.as_mut() {
                    fat.background = ColorAttribute::Default;
                }
                self.deallocate_fat_attributes_if_none();
            }
            ColorAttribute::PaletteIndex(idx) => {
                self.background = SmallColor::PaletteIndex(idx);
                if let Some(fat) = self.fat.as_mut() {
                    fat.background = ColorAttribute::Default;
                }
                self.deallocate_fat_attributes_if_none();
            }
            background => {
                self.background = SmallColor::Default;
                self.allocate_fat_attributes();
                self.fat.as_mut().unwrap().background = background;
            }
        }

        self
    }

    pub fn background(&self) -> ColorAttribute {
        if let Some(fat) = self.fat.as_ref() {
            if fat.background != ColorAttribute::Default {
                return fat.background;
            }
        }
        self.background.into()
    }

    fn allocate_fat_attributes(&mut self) {
        if self.fat.is_none() {
            self.fat.replace(Box::new(FatAttributes {
                hyperlink: None,
                image: vec![],
                underline_color: ColorAttribute::Default,
                foreground: ColorAttribute::Default,
                background: ColorAttribute::Default,
            }));
        }
    }

    fn deallocate_fat_attributes_if_none(&mut self) {
        let deallocate = self
            .fat
            .as_ref()
            .map(|fat| {
                fat.image.is_empty()
                    && fat.hyperlink.is_none()
                    && fat.underline_color == ColorAttribute::Default
                    && fat.foreground == ColorAttribute::Default
                    && fat.background == ColorAttribute::Default
            })
            .unwrap_or(false);
        if deallocate {
            self.fat.take();
        }
    }

    pub fn set_hyperlink(&mut self, link: Option<Arc<Hyperlink>>) -> &mut Self {
        if link.is_none() && self.fat.is_none() {
            self
        } else {
            self.allocate_fat_attributes();
            self.fat.as_mut().unwrap().hyperlink = link;
            self.deallocate_fat_attributes_if_none();
            self
        }
    }

    /// Assign a single image to a cell.
    pub fn set_image(&mut self, image: Box<ImageCell>) -> &mut Self {
        self.allocate_fat_attributes();
        self.fat.as_mut().unwrap().image = vec![image];
        self
    }

    /// Clear all images from a cell
    pub fn clear_images(&mut self) -> &mut Self {
        if let Some(fat) = self.fat.as_mut() {
            fat.image.clear();
        }
        self.deallocate_fat_attributes_if_none();
        self
    }

    pub fn detach_image_with_placement(&mut self, image_id: u32, placement_id: Option<u32>) {
        if let Some(fat) = self.fat.as_mut() {
            fat.image
                .retain(|im| !im.matches_placement(image_id, placement_id));
        }
        self.deallocate_fat_attributes_if_none();
    }

    /// Add an image attachement, preserving any existing attachments.
    /// The list of images is maintained in z-index order
    pub fn attach_image(&mut self, image: Box<ImageCell>) -> &mut Self {
        self.allocate_fat_attributes();
        let fat = self.fat.as_mut().unwrap();
        let z_index = image.z_index();
        match fat
            .image
            .binary_search_by(|probe| probe.z_index().cmp(&z_index))
        {
            Ok(idx) | Err(idx) => fat.image.insert(idx, image),
        }
        self
    }

    pub fn set_underline_color<C: Into<ColorAttribute>>(
        &mut self,
        underline_color: C,
    ) -> &mut Self {
        let underline_color = underline_color.into();
        if underline_color == ColorAttribute::Default && self.fat.is_none() {
            self
        } else {
            self.allocate_fat_attributes();
            self.fat.as_mut().unwrap().underline_color = underline_color;
            self.deallocate_fat_attributes_if_none();
            self
        }
    }

    /// Clone the attributes, but exclude fancy extras such
    /// as hyperlinks or future sprite things
    pub fn clone_sgr_only(&self) -> Self {
        let mut res = Self {
            attributes: self.attributes,
            foreground: self.foreground,
            background: self.background,
            fat: None,
        };
        if let Some(fat) = self.fat.as_ref() {
            if fat.background != ColorAttribute::Default
                || fat.foreground != ColorAttribute::Default
            {
                res.allocate_fat_attributes();
                let mut new_fat = res.fat.as_mut().unwrap();
                new_fat.foreground = fat.foreground;
                new_fat.background = fat.background;
            }
        }
        // Reset the semantic type; clone_sgr_only is used primarily
        // to create a "blank" cell when clearing and we want that to
        // be deterministically tagged as Output so that we have an
        // easier time in get_semantic_zones.
        res.set_semantic_type(SemanticType::default());
        res.set_underline_color(self.underline_color());
        res
    }

    pub fn hyperlink(&self) -> Option<&Arc<Hyperlink>> {
        self.fat.as_ref().and_then(|fat| fat.hyperlink.as_ref())
    }

    /// Returns the list of attached images in z-index order.
    /// Returns None if there are no attached images; will
    /// never return Some(vec![]).
    pub fn images(&self) -> Option<Vec<ImageCell>> {
        let fat = self.fat.as_ref()?;
        if fat.image.is_empty() {
            return None;
        }
        Some(fat.image.iter().map(|im| im.as_ref().clone()).collect())
    }

    pub fn underline_color(&self) -> ColorAttribute {
        self.fat
            .as_ref()
            .map(|fat| fat.underline_color)
            .unwrap_or(ColorAttribute::Default)
    }
}

#[cfg(feature = "use_serde")]
fn deserialize_teenystring<'de, D>(deserializer: D) -> Result<TeenyString, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    Ok(TeenyString::from_str(&text, None))
}

#[cfg(feature = "use_serde")]
fn serialize_teenystring<S>(value: &TeenyString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // unsafety: this is safe because the Cell constructor guarantees
    // that the storage is valid utf8
    let s = unsafe { std::str::from_utf8_unchecked(value.as_bytes()) };
    s.serialize(serializer)
}

/// TeenyString encodes string storage in a single machine word.
/// The scheme is simple but effective: strings that encode into a
/// byte slice that is 1 less byte than the machine word size can
/// be encoded directly into the usize bits stored in the struct.
/// A marker bit (LSB for big endian, MSB for little endian) is
/// set to indicate that the string is stored inline.
/// If the string is longer than this then a `Vec<u8>` is allocated
/// from the heap and the usize holds its raw pointer address.
///
/// When the string is inlined, the next-MSB is used to short-cut
/// calling grapheme_column_width; if it is set, then the TeenyString
/// has length 2, otherwise, it has length 1 (we don't allow zero-length
/// strings).
struct TeenyString(usize);
struct TeenyStringHeap {
    bytes: Vec<u8>,
    width: usize,
}

impl TeenyString {
    const fn marker_mask() -> usize {
        if cfg!(target_endian = "little") {
            cfg_if::cfg_if! {
                if #[cfg(target_pointer_width = "64")] {
                    0x80000000_00000000
                } else if #[cfg(target_pointer_width = "32")] {
                    0x80000000
                } else if #[cfg(target_pointer_width = "16")] {
                    0x8000
                } else {
                    panic!("unsupported target");
                }
            }
        } else {
            0x1
        }
    }

    const fn double_wide_mask() -> usize {
        if cfg!(target_endian = "little") {
            cfg_if::cfg_if! {
                if #[cfg(target_pointer_width = "64")] {
                    0xc0000000_00000000
                } else if #[cfg(target_pointer_width = "32")] {
                    0xc0000000
                } else if #[cfg(target_pointer_width = "16")] {
                    0xc000
                } else {
                    panic!("unsupported target");
                }
            }
        } else {
            0x3
        }
    }

    const fn is_marker_bit_set(word: usize) -> bool {
        let mask = Self::marker_mask();
        word & mask == mask
    }

    const fn is_double_width(word: usize) -> bool {
        let mask = Self::double_wide_mask();
        word & mask == mask
    }

    const fn set_marker_bit(word: usize, width: usize) -> usize {
        if width > 1 {
            word | Self::double_wide_mask()
        } else {
            word | Self::marker_mask()
        }
    }

    pub fn from_str(s: &str, width: Option<usize>) -> Self {
        // De-fang the input text such that it has no special meaning
        // to a terminal.  All control and movement characters are rewritten
        // as a space.
        let s = if s.is_empty() || s == "\r\n" {
            " "
        } else if s.len() == 1 {
            let b = s.as_bytes()[0];
            if b < 0x20 || b == 0x7f {
                " "
            } else {
                s
            }
        } else {
            s
        };

        let bytes = s.as_bytes();
        let len = bytes.len();
        let width = width.unwrap_or_else(|| grapheme_column_width(s, None));

        if len < std::mem::size_of::<usize>() {
            debug_assert!(width < 3);

            let mut word = 0usize;
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    &mut word as *mut usize as *mut u8,
                    len,
                );
            }
            let word = Self::set_marker_bit(word, width);
            Self(word)
        } else {
            let vec = Box::new(TeenyStringHeap {
                bytes: bytes.to_vec(),
                width,
            });
            let ptr = Box::into_raw(vec);
            Self(ptr as usize)
        }
    }

    pub const fn space() -> Self {
        Self(if cfg!(target_endian = "little") {
            cfg_if::cfg_if! {
                if #[cfg(target_pointer_width = "64")] {
                    0x80000000_00000020
                } else if #[cfg(target_pointer_width = "32")] {
                    0x80000020
                } else if #[cfg(target_pointer_width = "16")] {
                    0x8020
                } else {
                    panic!("unsupported target");
                }
            }
        } else {
            cfg_if::cfg_if! {
                if #[cfg(target_pointer_width = "64")] {
                    0x20000000_00000001
                } else if #[cfg(target_pointer_width = "32")] {
                    0x20000001
                } else if #[cfg(target_pointer_width = "16")] {
                    0x2001
                } else {
                    panic!("unsupported target");
                }
            }
        })
    }

    pub fn from_char(c: char) -> Self {
        let mut bytes = [0u8; 8];
        Self::from_str(c.encode_utf8(&mut bytes), None)
    }

    pub fn width(&self) -> usize {
        if Self::is_marker_bit_set(self.0) {
            if Self::is_double_width(self.0) {
                2
            } else {
                1
            }
        } else {
            let heap = self.0 as *const usize as *const TeenyStringHeap;
            unsafe { (*heap).width }
        }
    }

    pub fn str(&self) -> &str {
        // unsafety: this is safe because the constructor guarantees
        // that the storage is valid utf8
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        if Self::is_marker_bit_set(self.0) {
            let bytes = &self.0 as *const usize as *const u8;
            let bytes =
                unsafe { std::slice::from_raw_parts(bytes, std::mem::size_of::<usize>() - 1) };
            let len = bytes
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(std::mem::size_of::<usize>() - 1);

            &bytes[0..len]
        } else {
            let heap = self.0 as *const usize as *const TeenyStringHeap;
            unsafe { (*heap).bytes.as_slice() }
        }
    }
}

impl Drop for TeenyString {
    fn drop(&mut self) {
        if !Self::is_marker_bit_set(self.0) {
            let vec = unsafe { Box::from_raw(self.0 as *mut usize as *mut Vec<u8>) };
            drop(vec);
        }
    }
}

impl std::clone::Clone for TeenyString {
    fn clone(&self) -> Self {
        if Self::is_marker_bit_set(self.0) {
            Self(self.0)
        } else {
            Self::from_str(self.str(), None)
        }
    }
}

impl std::cmp::PartialEq for TeenyString {
    fn eq(&self, rhs: &Self) -> bool {
        self.as_bytes().eq(rhs.as_bytes())
    }
}
impl std::cmp::Eq for TeenyString {}

/// Models the contents of a cell on the terminal display
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq)]
pub struct Cell {
    #[cfg_attr(
        feature = "use_serde",
        serde(
            deserialize_with = "deserialize_teenystring",
            serialize_with = "serialize_teenystring"
        )
    )]
    text: TeenyString,
    attrs: CellAttributes,
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("Cell")
            .field("text", &self.str())
            .field("width", &self.width())
            .field("attrs", &self.attrs)
            .finish()
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::blank()
    }
}

impl Cell {
    /// Create a new cell holding the specified character and with the
    /// specified cell attributes.
    /// All control and movement characters are rewritten as a space.
    pub fn new(text: char, attrs: CellAttributes) -> Self {
        let storage = TeenyString::from_char(text);
        Self {
            text: storage,
            attrs,
        }
    }

    pub const fn blank() -> Self {
        Self {
            text: TeenyString::space(),
            attrs: CellAttributes::blank(),
        }
    }

    pub const fn blank_with_attrs(attrs: CellAttributes) -> Self {
        Self {
            text: TeenyString::space(),
            attrs,
        }
    }

    /// Indicates whether this cell has text or emoji presentation.
    /// The width already reflects that choice; this information
    /// is also useful when selecting an appropriate font.
    pub fn presentation(&self) -> Presentation {
        match Presentation::for_grapheme(self.str()) {
            (_, Some(variation)) => variation,
            (presentation, None) => presentation,
        }
    }

    /// Create a new cell holding the specified grapheme.
    /// The grapheme is passed as a string slice and is intended to hold
    /// double-width characters, or combining unicode sequences, that need
    /// to be treated as a single logical "character" that can be cursored
    /// over.  This function technically allows for an arbitrary string to
    /// be passed but it should not be used to hold strings other than
    /// graphemes.
    pub fn new_grapheme(text: &str, attrs: CellAttributes) -> Self {
        let storage = TeenyString::from_str(text, None);

        Self {
            text: storage,
            attrs,
        }
    }

    pub fn new_grapheme_with_width(text: &str, width: usize, attrs: CellAttributes) -> Self {
        let storage = TeenyString::from_str(text, Some(width));
        Self {
            text: storage,
            attrs,
        }
    }

    /// Returns the textual content of the cell
    pub fn str(&self) -> &str {
        self.text.str()
    }

    /// Returns the number of cells visually occupied by this grapheme
    pub fn width(&self) -> usize {
        self.text.width()
    }

    /// Returns the attributes of the cell
    pub fn attrs(&self) -> &CellAttributes {
        &self.attrs
    }

    pub fn attrs_mut(&mut self) -> &mut CellAttributes {
        &mut self.attrs
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnicodeVersion(pub u8);

pub const LATEST_UNICODE_VERSION: UnicodeVersion = UnicodeVersion(14);

/// Returns the number of cells visually occupied by a sequence
/// of graphemes.
/// Calls through to `grapheme_column_width` for each grapheme
/// and sums up the length.
pub fn unicode_column_width(s: &str, version: Option<UnicodeVersion>) -> usize {
    use unicode_segmentation::UnicodeSegmentation;
    s.graphemes(true)
        .map(|g| grapheme_column_width(g, version))
        .sum()
}

/// Returns the number of cells visually occupied by a grapheme.
/// The input string must be a single grapheme.
///
/// There are some frustrating dragons in the realm of terminal cell widths:
///
/// a) wcwidth and wcswidth are widely used by applications and may be
///    several versions of unicode behind the current version
/// b) The width of characters has and will change in the future.
///    Unicode Version 8 -> 9 made some characters wider.
///    Unicode 14 defines Emoji variation selectors that change the
///    width depending on trailing context in the unicode sequence.
///
/// Differing opinions about the width leads to visual artifacts in
/// text and and line editors, especially with respect to cursor placement.
///
/// There aren't any really great solutions to this problem, as a given
/// terminal emulator may be fine locally but essentially breaks when
/// ssh'ing into a remote system with a divergent wcwidth implementation.
///
/// This means that a global understanding of the unicode version that
/// is in use isn't a good solution.
///
/// The approach that wezterm wants to take here is to define a
/// configuration value that sets the starting level of unicode conformance,
/// and to define an escape sequence that can push/pop a desired confirmance
/// level onto a stack maintained by the terminal emulator.
///
/// The terminal emulator can then pass the unicode version through to
/// the Cell that is used to hold a grapheme, and that per-Cell version
/// can then be used to calculate width.
pub fn grapheme_column_width(s: &str, version: Option<UnicodeVersion>) -> usize {
    let version = version.unwrap_or(LATEST_UNICODE_VERSION).0;

    let width: usize = s
        .chars()
        .map(|c| {
            let c = WcWidth::from_char(c);
            if version >= 9 {
                c.width_unicode_9_or_later()
            } else {
                c.width_unicode_8_or_earlier()
            }
        })
        .sum::<u8>()
        .into();

    if version >= 14 {
        match Presentation::for_grapheme(s) {
            (_, Some(Presentation::Emoji)) => 2,
            (_, Some(Presentation::Text)) => 1,
            (Presentation::Emoji, None) => 2,
            (Presentation::Text, None) => width.min(2),
        }
    } else {
        width.min(2)
    }
}

/// Models a change in the attributes of a cell in a stream of changes.
/// Each variant specifies one of the possible attributes; the corresponding
/// value holds the new value to be used for that attribute.
#[cfg_attr(feature = "use_serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AttributeChange {
    Intensity(Intensity),
    Underline(Underline),
    Italic(bool),
    Blink(Blink),
    Reverse(bool),
    StrikeThrough(bool),
    Invisible(bool),
    Foreground(ColorAttribute),
    Background(ColorAttribute),
    Hyperlink(Option<Arc<Hyperlink>>),
}

#[cfg(test)]
mod test {
    use super::*;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn teeny_string() {
        let s = TeenyString::from_char('a');
        assert_eq!(s.as_bytes(), &[b'a']);

        let longer = TeenyString::from_str("hellothere", None);
        assert_eq!(longer.as_bytes(), b"hellothere");

        assert_eq!(
            TeenyString::from_char(' ').as_bytes(),
            TeenyString::space().as_bytes()
        );
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn memory_usage() {
        assert_eq!(std::mem::size_of::<crate::color::RgbColor>(), 4);
        assert_eq!(std::mem::size_of::<ColorAttribute>(), 8);
        assert_eq!(std::mem::size_of::<CellAttributes>(), 16);
        assert_eq!(std::mem::size_of::<Cell>(), 24);
        assert_eq!(std::mem::size_of::<Vec<u8>>(), 24);
        assert_eq!(std::mem::size_of::<char>(), 4);
        assert_eq!(std::mem::size_of::<TeenyString>(), 8);
    }

    #[test]
    fn nerf_special() {
        for c in " \n\r\t".chars() {
            let cell = Cell::new(c, CellAttributes::default());
            assert_eq!(cell.str(), " ");
        }

        for g in &["", " ", "\n", "\r", "\t", "\r\n"] {
            let cell = Cell::new_grapheme(g, CellAttributes::default());
            assert_eq!(cell.str(), " ");
        }
    }

    #[test]
    fn test_width() {
        let foot = "\u{1f9b6}";
        eprintln!("foot chars");
        for c in foot.chars() {
            eprintln!("char: {:?}", c);
        }
        assert_eq!(unicode_column_width(foot, None), 2, "{} should be 2", foot);

        let women_holding_hands_dark_skin_tone_medium_light_skin_tone =
            "\u{1F469}\u{1F3FF}\u{200D}\u{1F91D}\u{200D}\u{1F469}\u{1F3FC}";

        // Ensure that we can hold this longer grapheme sequence in the cell
        // and correctly return its string contents!
        let cell = Cell::new_grapheme(
            women_holding_hands_dark_skin_tone_medium_light_skin_tone,
            CellAttributes::default(),
        );
        assert_eq!(
            cell.str(),
            women_holding_hands_dark_skin_tone_medium_light_skin_tone
        );
        assert_eq!(
            cell.width(),
            2,
            "width of {} should be 2",
            women_holding_hands_dark_skin_tone_medium_light_skin_tone
        );

        let deaf_man = "\u{1F9CF}\u{200D}\u{2642}\u{FE0F}";
        eprintln!("deaf_man chars");
        for c in deaf_man.chars() {
            eprintln!("char: {:?}", c);
        }
        assert_eq!(unicode_column_width(deaf_man, None), 2);

        let man_dancing = "\u{1F57A}";
        assert_eq!(
            unicode_column_width(man_dancing, Some(UnicodeVersion(9))),
            2
        );
        assert_eq!(
            unicode_column_width(man_dancing, Some(UnicodeVersion(8))),
            1
        );

        // This is a codepoint in the private use area
        let font_awesome_star = "\u{f005}";
        eprintln!("font_awesome_star {}", font_awesome_star.escape_debug());
        assert_eq!(unicode_column_width(font_awesome_star, None), 1);

        let england_flag = "\u{1f3f4}\u{e0067}\u{e0062}\u{e0065}\u{e006e}\u{e0067}\u{e007f}";
        assert_eq!(unicode_column_width(england_flag, None), 2);
    }

    #[test]
    fn issue_1161() {
        let x_ideographic_space_x = "x\u{3000}x";
        assert_eq!(unicode_column_width(x_ideographic_space_x, None), 4);
        assert_eq!(
            x_ideographic_space_x.graphemes(true).collect::<Vec<_>>(),
            vec!["x".to_string(), "\u{3000}".to_string(), "x".to_string()],
        );

        let c = Cell::new_grapheme("\u{3000}", CellAttributes::blank());
        assert_eq!(c.width(), 2);
    }

    #[test]
    fn issue_997() {
        let victory_hand = "\u{270c}";
        let victory_hand_text_presentation = "\u{270c}\u{fe0e}";

        assert_eq!(
            unicode_column_width(victory_hand_text_presentation, None),
            1
        );
        assert_eq!(unicode_column_width(victory_hand, None), 1);

        assert_eq!(
            victory_hand_text_presentation
                .graphemes(true)
                .collect::<Vec<_>>(),
            vec![victory_hand_text_presentation.to_string()]
        );
        assert_eq!(
            victory_hand.graphemes(true).collect::<Vec<_>>(),
            vec![victory_hand.to_string()]
        );

        let copyright_emoji_presentation = "\u{00A9}\u{FE0F}";
        assert_eq!(
            copyright_emoji_presentation
                .graphemes(true)
                .collect::<Vec<_>>(),
            vec![copyright_emoji_presentation.to_string()]
        );
        assert_eq!(unicode_column_width(copyright_emoji_presentation, None), 2);
        assert_eq!(
            unicode_column_width(copyright_emoji_presentation, Some(UnicodeVersion(9))),
            1
        );

        let copyright_text_presentation = "\u{00A9}";
        assert_eq!(
            copyright_text_presentation
                .graphemes(true)
                .collect::<Vec<_>>(),
            vec![copyright_text_presentation.to_string()]
        );
        assert_eq!(unicode_column_width(copyright_text_presentation, None), 1);

        let raised_fist = "\u{270a}";
        // Not valid to have explicit Text presentation for raised fist
        let raised_fist_text = "\u{270a}\u{fe0e}";
        assert_eq!(
            Presentation::for_grapheme(raised_fist),
            (Presentation::Emoji, None)
        );
        assert_eq!(unicode_column_width(raised_fist, None), 2);
        assert_eq!(
            Presentation::for_grapheme(raised_fist_text),
            (Presentation::Emoji, None)
        );
        assert_eq!(unicode_column_width(raised_fist_text, None), 2);

        assert_eq!(
            raised_fist_text.graphemes(true).collect::<Vec<_>>(),
            vec![raised_fist_text.to_string()]
        );
        assert_eq!(
            raised_fist.graphemes(true).collect::<Vec<_>>(),
            vec![raised_fist.to_string()]
        );
    }
}
