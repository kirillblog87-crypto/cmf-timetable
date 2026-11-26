//! LVGL screens matching the supplied video: lesson cards, lesson editor, subject picker and live lesson clock.
use cmf_wp2_app::lv::*;
use cmf_wp2_app::stock;
use crate::model::{Lesson, Timetable, SUBJECTS};

const ACCENT: i32 = 0xC7B8FF;
const CARD: i32 = 0x211F2A;
const MUTED: i32 = 0x92909D;
pub const FONT_SMALL: u8 = 1;
pub const FONT_TIME: u8 = 2;

#[derive(Clone, Copy)]
pub struct Screen { pub page: Obj, pub actions: [Obj; 10], pub count: usize, pub keys: [Obj; 40], pub key_count: usize, pub progress: Obj, pub clock: Obj, pub detail: Obj, pub countdown: Obj }
impl Screen { pub const fn empty() -> Self { Self { page: ROOT, actions: [ROOT;10], count:0, keys:[ROOT;40], key_count:0, progress:ROOT, clock:ROOT, detail:ROOT, countdown:ROOT } } }

pub fn fonts(b: &mut Batch) -> Result<(), LvError> {
    stock::font(b)?; b.font(FONT_SMALL, StockFont::F18M)?; b.font(FONT_TIME, StockFont::F48M)
}

fn page(b: &mut Batch, title: &str, subtitle: &str) -> Result<(Obj, Obj), LvError> {
    stock::theme(b)?;
    let p = b.create(Class::Obj, ROOT)?; b.size(p, stock::SCREEN, stock::SCREEN)?;
    for prop in [Prop::PadTop,Prop::PadBottom,Prop::PadLeft,Prop::PadRight,Prop::BorderWidth,Prop::BgOpa] { b.style(p,prop,0,0)?; }
    b.flags(p, &[], &[Flag::Scrollable])?;
    let t = b.create(Class::Label,p)?; b.size(t,330,34)?; b.align(t,Align::TopMid,0,38)?;
    b.style(t,Prop::TextAlign,0,2)?; b.style(t,Prop::TextFont,0,stock::FONT as i32)?; b.text(t,title)?;
    let sub = b.create(Class::Label,p)?; b.size(sub,340,22)?; b.align(sub,Align::TopMid,0,72)?;
    b.style(sub,Prop::TextAlign,0,2)?; b.style(sub,Prop::TextColor,0,MUTED)?; b.style(sub,Prop::TextFont,0,FONT_SMALL as i32)?; b.text(sub,subtitle)?;
    Ok((p,t))
}

fn label(b: &mut Batch, parent: Obj, text: &str, y: i16, width: i32, height: i32, color: i32, font: u8) -> Result<Obj,LvError> {
    let o=b.create(Class::Label,parent)?; b.size(o,width,height)?; b.align(o,Align::TopMid,0,y)?;
    b.style(o,Prop::TextAlign,0,2)?; b.style(o,Prop::TextColor,0,color)?; b.style(o,Prop::TextFont,0,font as i32)?; b.text(o,text)?; Ok(o)
}

fn card(b: &mut Batch, parent: Obj, text: &str, y: i16, h: i32, width: i32) -> Result<Obj,LvError> {
    let o=b.create(Class::Obj,parent)?; b.size(o,width,h)?; b.align(o,Align::TopMid,0,y)?;
    b.style(o,Prop::Radius,0,17)?; b.style(o,Prop::BgColor,0,CARD)?; b.style(o,Prop::BgOpa,0,255)?;
    b.style(o,Prop::BorderWidth,0,0)?; b.flags(o,&[Flag::Clickable],&[Flag::Scrollable])?; b.events(o,&[Ev::Clicked])?;
    let l=b.create(Class::Label,o)?; b.size(l,width-24,h-8)?; b.align(l,Align::Center,0,0)?;
    b.style(l,Prop::TextAlign,0,0)?; b.style(l,Prop::TextColor,0,stock::WHITE)?; b.style(l,Prop::TextFont,0,FONT_SMALL as i32)?; b.text(l,text)?;
    Ok(o)
}

fn pill(b: &mut Batch, parent: Obj, text: &str, y: i16, width: i32) -> Result<Obj,LvError> {
    let o=b.create(Class::Obj,parent)?; b.size(o,width,48)?; b.align(o,Align::TopMid,0,y)?;
    b.style(o,Prop::Radius,0,24)?; b.style(o,Prop::BgColor,0,ACCENT)?; b.style(o,Prop::BgOpa,0,255)?;
    b.style(o,Prop::BorderWidth,0,0)?; b.flags(o,&[Flag::Clickable],&[Flag::Scrollable])?; b.events(o,&[Ev::Clicked])?;
    let l=b.create(Class::Label,o)?; b.align(l,Align::Center,0,0)?; b.style(l,Prop::TextColor,0,0x17151F)?; b.style(l,Prop::TextFont,0,FONT_SMALL as i32)?; b.text(l,text)?; Ok(o)
}

fn hm(minute:u16, out:&mut [u8;5])->&str { crate::model::time_text(minute,out) }

pub fn list(b:&mut Batch, t:&Timetable, offset:usize, day:&str)->Result<Screen,LvError>{
    let (p,_)=page(b,"Расписание",day)?; let mut s=Screen{page:p,..Screen::empty()};
    let mut y=108i16;
    for i in offset..(offset+4).min(t.len as usize){
        let l=t.lessons[i]; let mut a=[0;5];let mut z=[0;5];let mut text=[0u8;48];
        let st=hm(l.start,&mut a);let en=hm(l.end,&mut z);
        let mut n=0; for part in [st," - ",en,"\n",l.title()] { for &c in part.as_bytes(){if n<text.len(){text[n]=c;n+=1;}} }
        let line=core::str::from_utf8(&text[..n]).unwrap_or("Урок");
        s.actions[s.count]=card(b,p,line,y,54,340)?;s.count+=1;y+=59;
    }
    s.actions[s.count]=pill(b,p,"СЕЙЧАС УРОК",y+2,220)?;s.count+=1;
    label(b,p,"Поверни колесо для прокрутки",y+55,300,20,MUTED,FONT_SMALL)?;
    Ok(s)
}

pub fn edit(b:&mut Batch,t:&Timetable,index:usize,field:u8)->Result<Screen,LvError>{
    let l=t.lesson(index).unwrap_or(Lesson::EMPTY);
    let hint=match field {0=>"Колесо: час начала",1=>"Колесо: минуты начала",2=>"Колесо: час окончания",_=>"Колесо: минуты окончания"};
    let (p,_)=page(b,"Урок",hint)?;
    let mut s=Screen{page:p,..Screen::empty()};
    let mut a=[0;5];let mut z=[0;5];let st=hm(l.start,&mut a);let en=hm(l.end,&mut z);
    let mut start=[0u8;32];let mut n=0;for part in ["НАЧАЛО УРОКА\n",st]{for &c in part.as_bytes(){start[n]=c;n+=1;}}
    let mut end=[0u8;32];let mut m=0;for part in ["КОНЕЦ УРОКА\n",en]{for &c in part.as_bytes(){end[m]=c;m+=1;}}
    let start_text=core::str::from_utf8(&start[..n]).unwrap_or("НАЧАЛО");
    let end_text=core::str::from_utf8(&end[..m]).unwrap_or("КОНЕЦ");
    let x=card(b,p,start_text,112,68,340)?;s.actions[0]=x;
    let y=card(b,p,end_text,190,68,340)?;s.actions[1]=y;
    let subject=card(b,p,l.title(),258,48,340)?;s.actions[2]=subject;
    s.actions[3]=pill(b,p,"СВОЁ НАЗВАНИЕ",316,240)?;
    s.actions[4]=pill(b,p,"ГОТОВО",374,210)?;s.count=5;
    Ok(s)
}


/// Full on-watch Cyrillic keyboard. Each key is a real LVGL button; no phone/system keyboard required.
pub fn keyboard(b:&mut Batch, current:&str)->Result<Screen,LvError>{
    let (p,_)=page(b,"Своё название","Нажимай буквы на экране")?;
    let mut s=Screen{page:p,..Screen::empty()};
    let preview=if current.is_empty(){"Введите название урока"}else{current};
    let preview_label=label(b,p,preview,104,350,34,ACCENT,FONT_SMALL)?;
    b.long_mode(preview_label,LongMode::Dot)?;
    let rows:[&[&str];3]=[
        &["й","ц","у","к","е","н","г","ш","щ","з","х"],
        &["ф","ы","в","а","п","р","о","л","д","ж","э"],
        &["я","ч","с","м","и","т","ь","б","ю","ё","ъ"],
    ];
    for (r,row) in rows.iter().enumerate(){
        let width=27i16;let gap=3i16;let total=row.len() as i16*(width+gap)-gap;
        for (i,ch) in row.iter().enumerate(){
            let key=b.create(Class::Btn,p)?;b.size(key,width as i32,42)?;
            let x=-(total/2)+(i as i16)*(width+gap)+width/2;
            b.align(key,Align::TopMid,x,150+r as i16*48)?;
            b.style(key,Prop::Radius,0,8)?;b.style(key,Prop::BgColor,0,CARD)?;b.style(key,Prop::BgOpa,0,255)?;b.style(key,Prop::BorderWidth,0,0)?;
            b.flags(key,&[Flag::Clickable],&[Flag::Scrollable])?;b.events(key,&[Ev::Clicked])?;
            let text=b.create(Class::Label,key)?;b.align(text,Align::Center,0,0)?;b.style(text,Prop::TextColor,0,stock::WHITE)?;b.style(text,Prop::TextFont,0,FONT_SMALL as i32)?;b.text(text,ch)?;
            s.keys[s.key_count]=key;s.key_count+=1;
        }
    }
    let controls=[("ПРОБЕЛ",100), ("СТЕРЕТЬ",82), ("ГОТОВО",100)];
    let mut x=-149i16;
    for (i,(caption,w)) in controls.iter().enumerate(){
        let key=b.create(Class::Btn,p)?;b.size(key,*w,42)?;b.align(key,Align::TopMid,x+(*w as i16)/2,300)?;
        b.style(key,Prop::Radius,0,12)?;b.style(key,Prop::BgColor,0,if i==2{ACCENT}else{CARD})?;b.style(key,Prop::BgOpa,0,255)?;b.style(key,Prop::BorderWidth,0,0)?;
        b.flags(key,&[Flag::Clickable],&[Flag::Scrollable])?;b.events(key,&[Ev::Clicked])?;
        let text=b.create(Class::Label,key)?;b.align(text,Align::Center,0,0)?;b.style(text,Prop::TextColor,0,if i==2{0x17151F}else{stock::WHITE})?;b.style(text,Prop::TextFont,0,FONT_SMALL as i32)?;b.text(text,caption)?;
        s.keys[s.key_count]=key;s.key_count+=1;x+=*w as i16+8;
    }
    let cancel=pill(b,p,"ОТМЕНА",358,170)?;s.actions[0]=cancel;s.count=1;
    Ok(s)
}

pub fn subjects(b:&mut Batch,offset:usize)->Result<Screen,LvError>{
    let (p,_)=page(b,"Выбери предмет","Нажми на нужный предмет")?;let mut s=Screen{page:p,..Screen::empty()};
    let mut y=104i16;
    for i in offset..(offset+7).min(SUBJECTS.len()) { s.actions[s.count]=card(b,p,SUBJECTS[i],y,32,340)?;s.count+=1;y+=35; }
    s.actions[s.count]=pill(b,p,"ЗАКРЫТЬ",y+5,180)?;s.count+=1;Ok(s)
}

pub fn live(b:&mut Batch,t:&Timetable,minute:u16,second:u8,clock:&str)->Result<Screen,LvError>{
    let (p,_)=page(b,"Расписание","Текущий урок")?;let mut s=Screen{page:p,..Screen::empty()};
    let current=t.current(minute);let next=t.next(minute);
    let (idx,l,caption)=if let Some((i,l))=current{(i,l,"СЕЙЧАС УРОК")}else if let Some((i,l))=next{(i,l,"СЛЕДУЮЩИЙ УРОК")}else{(0,Lesson::EMPTY,"УРОКОВ БОЛЬШЕ НЕТ")};
    let c=label(b,p,caption,116,300,24,ACCENT,FONT_SMALL)?;let _=c;
    s.clock=label(b,p,clock,154,340,64,stock::WHITE,FONT_TIME)?;
    let mut a=[0;5];let mut z=[0;5];let mut buf=[0u8;64];let mut n=0usize;
    let detail=if current.is_none() && next.is_none(){
        "Все уроки на сегодня завершены"
    }else{
        let parts:[&str;3]=if current.is_some(){[l.title()," - до ",hm(l.end,&mut z)]}else{[l.title()," - начало ",hm(l.start,&mut a)]};
        for part in parts{for &ch in part.as_bytes(){if n<buf.len(){buf[n]=ch;n+=1;}}}
        core::str::from_utf8(&buf[..n]).unwrap_or("Урок")
    };
    s.detail=label(b,p,detail,224,360,26,stock::WHITE,FONT_SMALL)?;
    let arc=b.create(Class::Arc,p)?;b.size(arc,290,290)?;b.align(arc,Align::Center,0,16)?;b.flags(arc,&[],&[Flag::Clickable])?;
    b.value(arc,if current.is_some(){((minute.saturating_sub(l.start)) as i32*100/(l.end-l.start).max(1) as i32).clamp(0,100)}else{0},false)?;
    b.style(arc,Prop::ArcColor,0,0x292631)?;b.style(arc,Prop::ArcOpa,0,255)?;b.style(arc,Prop::ArcWidth,0,8)?;
    b.style(arc,Prop::ArcColor,stock::INDICATOR,ACCENT)?;b.style(arc,Prop::ArcOpa,stock::INDICATOR,255)?;b.style(arc,Prop::ArcWidth,stock::INDICATOR,8)?;s.progress=arc;
    s.countdown=label(b,p,"",282,320,26,MUTED,FONT_SMALL)?;
    s.actions[0]=pill(b,p,"НАЗАД К РАСПИСАНИЮ",370,250)?;s.count=1;let _=idx;
    live_update(b,&s,t,minute,second,clock)?; Ok(s)
}

pub fn live_update(b:&mut Batch,s:&Screen,t:&Timetable,minute:u16,second:u8,clock:&str)->Result<(),LvError>{
    b.text(s.clock,clock)?;
    let current=t.current(minute);let next=t.next(minute);
    let l=if let Some((_,l))=current{l}else if let Some((_,l))=next{l}else{return b.text(s.countdown,"Уроки закончились")};
    let remaining=if current.is_some(){l.end.saturating_sub(minute).saturating_mul(60).saturating_sub(second as u16)}else{l.start.saturating_sub(minute).saturating_mul(60).saturating_sub(second as u16)};
    let mins=remaining/60;let secs=remaining%60;
    let mut left=[0u8;40];let mut n=0;
    let prefix=if current.is_some(){"ДО КОНЦА УРОКА "}else{"ДО НАЧАЛА УРОКА "};
    for &ch in prefix.as_bytes(){left[n]=ch;n+=1;}
    if mins>=100 { left[n]=b'0'+(mins/100) as u8;n+=1; }
    left[n]=b'0'+((mins/10)%10) as u8;n+=1;left[n]=b'0'+(mins%10) as u8;n+=1;left[n]=b':';n+=1;
    left[n]=b'0'+(secs/10) as u8;n+=1;left[n]=b'0'+(secs%10) as u8;n+=1;
    b.text(s.countdown,core::str::from_utf8(&left[..n]).unwrap_or(""))?;
    let value=if current.is_some(){((minute.saturating_sub(l.start)) as i32*60+second as i32)*100/((l.end-l.start).max(1) as i32*60)}else{0};
    b.value(s.progress,value.clamp(0,100),false)
}
