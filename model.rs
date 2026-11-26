//! Allocation-free timetable model with user-defined Cyrillic lesson names.
pub const MAX_LESSONS: usize = 8;
pub const SUBJECTS: [&str; 16] = [
    "Математика", "Русский язык", "Казахский язык", "Английский", "Информатика",
    "Физика", "Химия", "Биология", "География", "История России",
    "История Казахстана", "Всемирная история", "Обществознание", "Литература",
    "Физкультура", "Искусство",
];
pub const CUSTOM_SUBJECT: u8 = 255;
pub const NAME_CAP: usize = 32;
const MAGIC: [u8; 3] = *b"TT2";
const OLD_MAGIC: [u8; 3] = *b"TT1";
const LESSON_BYTES: usize = 1 + 2 + 2 + 1 + NAME_CAP;
pub const BLOB_LEN: usize = 4 + MAX_LESSONS * LESSON_BYTES;
pub const OLD_BLOB_LEN: usize = 4 + MAX_LESSONS * 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lesson { pub subject: u8, pub start: u16, pub end: u16, pub name: [u8; NAME_CAP], pub name_len: u8 }
impl Lesson {
    pub const EMPTY: Self = Self { subject: 0, start: 8 * 60, end: 8 * 60 + 45, name: [0; NAME_CAP], name_len: 0 };
    pub fn valid(self) -> bool {
        (self.subject < SUBJECTS.len() as u8 || self.subject == CUSTOM_SUBJECT)
            && self.name_len as usize <= NAME_CAP && self.start < self.end && self.end <= 1440
            && (self.subject != CUSTOM_SUBJECT || self.name_len > 0)
            && core::str::from_utf8(&self.name[..self.name_len as usize]).is_ok()
    }
    pub fn title(&self) -> &str {
        if self.subject == CUSTOM_SUBJECT && self.name_len > 0 {
            core::str::from_utf8(&self.name[..self.name_len as usize]).unwrap_or("Урок")
        } else { SUBJECTS.get(self.subject as usize).copied().unwrap_or("Урок") }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timetable { pub lessons: [Lesson; MAX_LESSONS], pub len: u8 }
impl Timetable {
    pub const fn new() -> Self {
        let mut lessons = [Lesson::EMPTY; MAX_LESSONS]; let mut i = 0;
        while i < MAX_LESSONS { let start = 8 * 60 + i as u16 * 55;
            lessons[i] = Lesson { subject: i as u8, start, end: start + 45, name: [0; NAME_CAP], name_len: 0 }; i += 1; }
        Self { lessons, len: MAX_LESSONS as u8 }
    }
    pub fn lesson(&self, index: usize) -> Option<Lesson> { (index < self.len as usize).then_some(self.lessons[index]) }
    pub fn set_subject(&mut self, index: usize, subject: usize) -> bool {
        if index >= self.len as usize || subject >= SUBJECTS.len() { return false; }
        let l=&mut self.lessons[index]; l.subject=subject as u8; l.name=[0;NAME_CAP]; l.name_len=0; true
    }
    pub fn set_custom_name(&mut self, index: usize, name: &[u8]) -> bool {
        if index >= self.len as usize || name.is_empty() || name.len() > NAME_CAP || core::str::from_utf8(name).is_err() { return false; }
        let l=&mut self.lessons[index]; l.name=[0;NAME_CAP]; l.name[..name.len()].copy_from_slice(name); l.name_len=name.len() as u8; l.subject=CUSTOM_SUBJECT; true
    }
    pub fn push_custom_char(&mut self, index: usize, ch: &str) -> bool {
        if index >= self.len as usize || ch.is_empty() || core::str::from_utf8(ch.as_bytes()).is_err() { return false; }
        let l=&mut self.lessons[index]; let n=l.name_len as usize;
        if n + ch.len() > NAME_CAP { return false; }
        l.name[n..n+ch.len()].copy_from_slice(ch.as_bytes()); l.name_len=(n+ch.len()) as u8; l.subject=CUSTOM_SUBJECT; true
    }
    /// Like `push_custom_char`, but the first letter of a name is capitalised (Cyrillic а-я, ё).
    pub fn push_typed_char(&mut self, index: usize, ch: &str) -> bool {
        let first = index < self.len as usize && self.lessons[index].name_len == 0;
        let mut buf = [0u8; 4];
        if first {
            let mut it = ch.chars();
            if let (Some(c), None) = (it.next(), it.next()) {
                let up = match c { '\u{0430}'..='\u{044F}' => char::from_u32(c as u32 - 0x20), '\u{0451}' => Some('\u{0401}'), _ => None };
                if let Some(u) = up { return self.push_custom_char(index, u.encode_utf8(&mut buf)); }
            }
        }
        self.push_custom_char(index, ch)
    }
    pub fn backspace_name(&mut self,index:usize)->bool {
        if index>=self.len as usize{return false;} let l=&mut self.lessons[index]; let mut n=l.name_len as usize;
        if n==0{return false;} while n>0 && (l.name[n-1]&0xC0)==0x80 {n-=1;} if n>0{n-=1;} l.name[n..].fill(0);l.name_len=n as u8;
        if n==0 && l.subject==CUSTOM_SUBJECT { l.subject=0; } true
    }
    pub fn clear_name(&mut self,index:usize)->bool { if index>=self.len as usize{return false;} let l=&mut self.lessons[index];l.name=[0;NAME_CAP];l.name_len=0;if l.subject==CUSTOM_SUBJECT{l.subject=0;}true }
    /// Field: 0=start hour, 1=start minute, 2=end hour, 3=end minute.
    pub fn adjust_time(&mut self,index:usize,field:u8,delta:i16)->bool {
        if index>=self.len as usize||field>3{return false;} let l=&mut self.lessons[index];let step=if field==1||field==3{5}else{60};
        let old=if field<=1{l.start}else{l.end};let next=(old as i32+delta as i32*step).clamp(0,1439) as u16;
        if field<=1{if next>=l.end{return false;}l.start=next;}else{if next<=l.start{return false;}l.end=next;}true
    }
    pub fn current(&self,minute:u16)->Option<(usize,Lesson)>{self.lessons[..self.len as usize].iter().copied().enumerate().find(|(_,l)|minute>=l.start&&minute<l.end)}
    pub fn next(&self,minute:u16)->Option<(usize,Lesson)>{self.lessons[..self.len as usize].iter().copied().enumerate().filter(|(_,l)|l.start>minute).min_by_key(|(_,l)|l.start)}
    pub fn encode(&self,out:&mut [u8;BLOB_LEN])->usize{
        out[..3].copy_from_slice(&MAGIC);out[3]=self.len.min(MAX_LESSONS as u8);
        for i in 0..MAX_LESSONS{let at=4+i*LESSON_BYTES;let l=self.lessons[i];out[at]=l.subject;out[at+1..at+3].copy_from_slice(&l.start.to_le_bytes());out[at+3..at+5].copy_from_slice(&l.end.to_le_bytes());out[at+5]=l.name_len;out[at+6..at+6+NAME_CAP].copy_from_slice(&l.name);} BLOB_LEN
    }
    pub fn decode(bytes:&[u8])->Option<Self>{
        if bytes.len()==OLD_BLOB_LEN && bytes[..3]==OLD_MAGIC {
            if bytes[3] as usize>MAX_LESSONS{return None;} let mut t=Self::new();t.len=bytes[3];
            for i in 0..MAX_LESSONS{let at=4+i*5;let l=&mut t.lessons[i];l.subject=bytes[at];l.start=u16::from_le_bytes([bytes[at+1],bytes[at+2]]);l.end=u16::from_le_bytes([bytes[at+3],bytes[at+4]]);if i<t.len as usize&&!l.valid(){return None;}}return Some(t)
        }
        if bytes.len()!=BLOB_LEN||bytes[..3]!=MAGIC||bytes[3] as usize>MAX_LESSONS{return None;}let mut t=Self::new();t.len=bytes[3];
        for i in 0..MAX_LESSONS{let at=4+i*LESSON_BYTES;let mut name=[0;NAME_CAP];name.copy_from_slice(&bytes[at+6..at+6+NAME_CAP]);let l=Lesson{subject:bytes[at],start:u16::from_le_bytes([bytes[at+1],bytes[at+2]]),end:u16::from_le_bytes([bytes[at+3],bytes[at+4]]),name_len:bytes[at+5],name};if i<t.len as usize&&!l.valid(){return None;}t.lessons[i]=l;}Some(t)
    }
}
pub fn time_text(minute:u16,out:&mut [u8;5])->&str{let h=(minute/60)%24;let m=minute%60;out[0]=b'0'+(h/10)as u8;out[1]=b'0'+(h%10)as u8;out[2]=b':';out[3]=b'0'+(m/10)as u8;out[4]=b'0'+(m%10)as u8;core::str::from_utf8(out).unwrap_or("00:00")}

#[cfg(test)] mod tests { use super::*;
#[test]fn default_has_eight_ordered_lessons(){let t=Timetable::new();assert_eq!(t.len,8);for i in 1..8{assert!(t.lessons[i].start>t.lessons[i-1].end);}}
#[test]fn current_and_next_are_found(){let t=Timetable::new();assert_eq!(t.current(8*60+2).unwrap().0,0);assert_eq!(t.current(8*60+45),None);assert_eq!(t.next(8*60+45).unwrap().0,1);}
#[test]fn editing_never_inverts_lesson(){let mut t=Timetable::new();assert!(!t.adjust_time(0,0,10));assert!(!t.adjust_time(0,2,-10));assert!(t.adjust_time(0,1,1));}
#[test]fn custom_cyrillic_name_and_storage_round_trip(){let mut t=Timetable::new();for c in ["О","р","ф","о","г","р","а","ф","и","я"]{assert!(t.push_custom_char(0,c));}assert_eq!(t.lessons[0].title(),"Орфография");let mut b=[0;BLOB_LEN];t.encode(&mut b);assert_eq!(Timetable::decode(&b),Some(t));}
#[test]fn backspace_removes_one_unicode_character(){let mut t=Timetable::new();t.set_custom_name(0,"Ёжик".as_bytes());t.backspace_name(0);assert_eq!(t.lessons[0].title(),"Ёжи");}
#[test]fn old_storage_is_migrated(){let t=Timetable::new();let mut b=[0;OLD_BLOB_LEN];b[..3].copy_from_slice(&OLD_MAGIC);b[3]=8;for i in 0..8{let at=4+i*5;b[at]=i as u8;let s=8*60+i as u16*55;b[at+1..at+3].copy_from_slice(&s.to_le_bytes());b[at+3..at+5].copy_from_slice(&(s+45).to_le_bytes());}let migrated=Timetable::decode(&b).unwrap();assert_eq!(migrated.len,t.len);}
#[test]fn first_typed_letter_is_capital(){let mut t=Timetable::new();t.set_custom_name(0,"".as_bytes());t.lessons[0].name_len=0;for c in ["р","ё","ж"]{assert!(t.push_typed_char(0,c));}assert_eq!(t.lessons[0].title(),"Рёж");}
#[test]fn clock_format_is_fixed_width(){let mut b=[0;5];assert_eq!(time_text(7*60+5,&mut b),"07:05");}}
