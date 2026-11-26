//! Timetable for CMF Watch Pro 2, following the supplied watch video.
#![cfg_attr(target_os = "none", no_std, no_main)]

#[cfg_attr(not(target_os = "none"), allow(dead_code))]
mod model;
#[cfg_attr(not(target_os = "none"), allow(dead_code))]
mod screen;

#[cfg(not(target_os = "none"))]
fn main() {}

#[cfg(target_os = "none")]
mod device {
    use crate::model::{Timetable, BLOB_LEN, SUBJECTS};
    use crate::screen::{self, Screen};
    use cmf_wp2_app::lv::{Batch, Lv};
    use cmf_wp2_app::{Api, BUTTON, CROWN, LV, PAUSE, RESUME};

    cmf_wp2_app::entry!(main);
    cmf_wp2_app::app_icon!();
    const CLICKED: i16 = 7;

    #[derive(Clone, Copy, PartialEq)]
    enum View { List, Edit, Subjects, Keyboard, Live }

    fn load(api: &Api) -> Timetable {
        let mut bytes = [0u8; BLOB_LEN];
        let n = api.load(&mut bytes);
        if n > 0 && (n as usize) <= BLOB_LEN { Timetable::decode(&bytes[..n as usize]).unwrap_or_else(Timetable::new) }
        else { Timetable::new() }
    }

    fn save(api: &Api, model: &Timetable) {
        let mut bytes = [0u8; BLOB_LEN]; model.encode(&mut bytes); api.save(&bytes);
    }

    fn clock(api: &Api, out: &mut [u8; 8]) -> Option<(u16,u8)> {
        let t = api.time()?;
        let h=t.hour.min(23);let m=t.minute.min(59);let s=t.second.min(59);
        out[0]=b'0'+h/10;out[1]=b'0'+h%10;out[2]=b':';out[3]=b'0'+m/10;out[4]=b'0'+m%10;
        out[5]=b':';out[6]=b'0'+s/10;out[7]=b'0'+s%10;
        Some((h as u16*60+m as u16,s))
    }

    fn show(api:&Api,lv:&Lv,b:&mut Batch,current:&mut Screen,view:View,model:&Timetable,
            offset:usize,lesson:usize,field:u8,subject_offset:usize)->bool {
        if current.page != cmf_wp2_app::lv::ROOT { let _=b.del(current.page); if b.submit(lv).is_err(){return false;} }
        let next=match view {
            View::List=>screen::list(b,model,offset,"Твои уроки на сегодня"),
            View::Edit=>screen::edit(b,model,lesson,field),
            View::Subjects=>screen::subjects(b,subject_offset),
            View::Keyboard=>{
                let l=model.lesson(lesson).unwrap_or(crate::model::Lesson::EMPTY);
                let name=if l.name_len>0 { core::str::from_utf8(&l.name[..l.name_len as usize]).unwrap_or("") } else { "" };
                screen::keyboard(b,name)
            }
            View::Live=>{
                let mut buf=[0u8;8];
                if let Some((minute,second))=clock(api,&mut buf) {
                    screen::live(b,model,minute,second,core::str::from_utf8(&buf).unwrap_or("--:--:--"))
                } else { screen::list(b,model,offset,"Установи время на часах") }
            }
        };
        match next { Ok(s)=>{*current=s;b.submit(lv).is_ok()},Err(_)=>false }
    }

    fn main(api:&'static Api)->i32 {
        let Some(lv)=api.lv() else{return -95};
        let Some(mut b)=Batch::new() else{return -1};
        if screen::fonts(&mut b).and_then(|_|b.submit(&lv)).is_err(){return 1;}
        let mut model=load(api);let mut view=View::List;let mut screen_now=Screen::empty();
        let (mut offset,mut lesson,mut field,mut subject_offset)=(0usize,0usize,0u8,0usize);
        if !show(api,&lv,&mut b,&mut screen_now,view,&model,offset,lesson,field,subject_offset){return 2;}
        let mut last_second=u8::MAX;let mut last_minute=u16::MAX;
        loop {
            let ev=api.event(if view==View::Live{200}else{u32::MAX});
            if let Some(e)=ev { match e.kind {
                BUTTON=>{save(api,&model);return 0;}
                CROWN if e.delta!=0=>{
                    match view {
                        View::List=>{let max=(model.len as usize).saturating_sub(4);offset=(offset as i32+e.delta.signum() as i32).clamp(0,max as i32) as usize;}
                        View::Subjects=>{let max=SUBJECTS.len().saturating_sub(7);subject_offset=(subject_offset as i32+e.delta.signum() as i32).clamp(0,max as i32) as usize;}
                        View::Edit=>{if model.adjust_time(lesson,field,e.delta){save(api,&model);}}
                        View::Live|View::Keyboard=>{}
                    }
                    if !show(api,&lv,&mut b,&mut screen_now,view,&model,offset,lesson,field,subject_offset){return 3;}
                }
                LV if e.y==CLICKED && e.x>=0=>{
                    let id=e.x as u16;
                    match view {
                        View::List=>{
                            if screen_now.count>0 && id==screen_now.actions[screen_now.count-1].0 {view=View::Live;}
                            else if let Some(i)=screen_now.actions[..screen_now.count.saturating_sub(1)].iter().position(|o|o.0==id){lesson=offset+i;field=0;view=View::Edit;}
                        }
                        View::Edit=>{
                            if id==screen_now.actions[0].0 {field=if field==0{1}else{0};}
                            else if id==screen_now.actions[1].0 {field=if field==2{3}else{2};}
                            else if id==screen_now.actions[2].0 {view=View::Subjects;subject_offset=0;}
                            else if id==screen_now.actions[3].0 {view=View::Keyboard;}
                            else if id==screen_now.actions[4].0 {view=View::List;}
                        }
                        View::Subjects=>{
                            if id==screen_now.actions[screen_now.count-1].0 {view=View::Edit;}
                            else if let Some(i)=screen_now.actions[..screen_now.count-1].iter().position(|o|o.0==id){model.set_subject(lesson,subject_offset+i);save(api,&model);view=View::Edit;}
                        }
                        View::Keyboard=>{
                            if id==screen_now.actions[0].0 {view=View::Edit;}
                            else if let Some(k)=screen_now.keys[..screen_now.key_count].iter().position(|o|o.0==id){
                                match k {
                                    0..=32=>{let alphabet=["й","ц","у","к","е","н","г","ш","щ","з","х","ф","ы","в","а","п","р","о","л","д","ж","э","я","ч","с","м","и","т","ь","б","ю","ё","ъ"];model.push_typed_char(lesson,alphabet[k]);save(api,&model);}
                                    33=>{model.push_custom_char(lesson," ");save(api,&model);}
                                    34=>{model.backspace_name(lesson);save(api,&model);}
                                    35=>{if model.lessons[lesson].name_len>0 {save(api,&model);}view=View::Edit;}
                                    _=>{}
                                }
                            }
                        }
                        View::Live=>{view=View::List;}
                    }
                    if !show(api,&lv,&mut b,&mut screen_now,view,&model,offset,lesson,field,subject_offset){return 4;}
                }
                PAUSE=>{
                    loop {match api.event(u32::MAX){Some(e) if e.kind==RESUME=>break,Some(e) if e.kind==BUTTON=>{save(api,&model);return 0;},_=>{}}}
                    last_second=u8::MAX;last_minute=u16::MAX;
                    if !show(api,&lv,&mut b,&mut screen_now,view,&model,offset,lesson,field,subject_offset){return 5;}
                }
                RESUME=>{last_second=u8::MAX;last_minute=u16::MAX;}
                _=>{}
            } }
            if view==View::Live {
                let mut buf=[0u8;8];
                if let Some((minute,sec))=clock(api,&mut buf) {
                    if minute!=last_minute {
                        last_minute=minute;last_second=sec;
                        if !show(api,&lv,&mut b,&mut screen_now,view,&model,offset,lesson,field,subject_offset){return 6;}
                    } else if sec!=last_second {
                        last_second=sec;
                        if screen::live_update(&mut b,&screen_now,&model,minute,sec,core::str::from_utf8(&buf).unwrap_or("--:--:--")).and_then(|_|b.submit(&lv)).is_err(){return 7;}
                    }
                }
            }
        }
    }
}
