#![no_std]

use cpuio::Port;

pub struct CMOS {
    century_handler: CMOSCenturyHandler,
    address_port: Port<u8>,
    data_port: Port<u8>,
}

impl CMOS {
    pub unsafe fn new(century_handler: CMOSCenturyHandler) -> CMOS {
        CMOS {
            century_handler,
            address_port: Port::<u8>::new(0x70),
            data_port: Port::<u8>::new(0x71),
        }
    }

    pub fn read(&mut self, output: &mut [u8; 128]) {
        for i in 0..128 {
            self.address_port.write(i);
            output[i as usize] = self.data_port.read();
        }
    }

    pub fn write(&mut self, input: &mut [u8; 128]) {
        for i in 0..128 {
            self.address_port.write(i);
            self.data_port.write(input[i as usize]);
        }
    }

    pub unsafe fn get_update_in_progress_flag(&mut self) -> u8 {
        self.address_port.write(0x0A);
        self.data_port.read() & 0x80
    }

    unsafe fn get_rtc_register(&mut self, reg: u8) -> u8 {
        self.address_port.write(reg);
        self.data_port.read()
    }

    unsafe fn update_into_rtc(&mut self, rtc_time: &mut RTCDateTime) {
        while self.get_update_in_progress_flag() != 0 {
            rtc_time.second = self.get_rtc_register(0x00);
            rtc_time.minute = self.get_rtc_register(0x02);
            rtc_time.hour = self.get_rtc_register(0x04);
            rtc_time.day = self.get_rtc_register(0x07);
            rtc_time.month = self.get_rtc_register(0x08);
            rtc_time.year = self.get_rtc_register(0x09) as usize;
        }
    }

    pub unsafe fn read_rtc(&mut self) -> RTCDateTime {

        let mut rtc_time = RTCDateTime {
            second: 0,
            minute: 0,
            hour: 0,
            day: 0,
            month: 0,
            year: 0,
        };

        self.update_into_rtc(&mut rtc_time);
    
        let mut century = 0;
        if let CMOSCenturyHandler::CenturyRegister(century_reg) = self.century_handler {
            century = self.get_rtc_register(century_reg);
        }

        let mut last_second;
        let mut last_minute;
        let mut last_hour;
        let mut last_day;
        let mut last_month;
        let mut last_year;
        let mut last_century;

        loop {
            last_second = rtc_time.second;
            last_minute = rtc_time.minute;
            last_hour = rtc_time.hour;
            last_day = rtc_time.day;
            last_month = rtc_time.month;
            last_year = rtc_time.year;
            last_century = century;
        
            self.update_into_rtc(&mut rtc_time);

            if last_second != rtc_time.second
            || last_minute != rtc_time.minute
            || last_hour != rtc_time.hour
            || last_day != rtc_time.day
            || last_month != rtc_time.month
            || last_year != rtc_time.year
            || last_century != century {
                break;
            }
        }

        let register_b = self.get_rtc_register(0x0B);

        if (register_b & 0x04) == 0 {
            rtc_time.second = (rtc_time.second & 0x0F) + ((rtc_time.second / 16) * 10);
            rtc_time.minute = (rtc_time.minute & 0x0F) + ((rtc_time.minute / 16) * 10);
            rtc_time.hour = ( (rtc_time.hour & 0x0F) + (((rtc_time.hour & 0x70) / 16) * 10) ) | (rtc_time.hour & 0x80);
            rtc_time.day = (rtc_time.day & 0x0F) + ((rtc_time.day / 16) * 10);
            rtc_time.month = (rtc_time.month & 0x0F) + ((rtc_time.month / 16) * 10);
            rtc_time.year = (rtc_time.year & 0x0F) + ((rtc_time.year / 16) * 10);

            if let CMOSCenturyHandler::CenturyRegister(_) = self.century_handler {
                century = (century & 0x0F) + ((century / 16) * 10);
            } 
        }

        if ((register_b & 0x02) == 0) && ((rtc_time.hour & 0x80) != 0) {
            rtc_time.hour = ((rtc_time.hour & 0x7F) + 12) % 24;
        }


        match self.century_handler {
            CMOSCenturyHandler::CenturyRegister(_) =>  rtc_time.year += (century * 100) as usize,
            CMOSCenturyHandler::CurrentYear(current_year) => {
                rtc_time.year += (current_year / 100) * 100;
                if rtc_time.year < current_year {
                    rtc_time.year += 100;
                }
            },
        }


        rtc_time
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CMOSCenturyHandler {
    CenturyRegister(u8),
    CurrentYear(usize),
}

#[derive(Debug, Clone, Copy)]
pub struct RTCDateTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: usize,
}
