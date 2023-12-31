static GET_DTC_COMMAND: &'static str = "03";
static CLEAR_DTC_COMMAND: &'static str = "04";
static GET_FREEZE_DTC_COMMAND: &'static str = "07";

fn hex_to_int(hex: &str) -> u8 {
	u8::from_str_radix(hex, 16).unwrap()
}

fn decrypt_dtc_code<'a>(code: &'a str) -> Vec<&'a str> {
	let mut dtc: Vec<&'a str> = Vec::new();
	let mut current = code;

	for _ in 0..3 {
		if current.len() < 4 {
			panic!("Tried to decode bad DTC: {}", code);
		}
		let tc = hex_to_int(&current[0..1]); //typecode
		let tc = tc >> 2;
		let type_code: &'a str;
		match tc {
			0 => type_code = "P",
			1 => type_code = "C",
			2 => type_code = "B",
			3 => type_code = "U",
			_ => panic!("Unknown type code: {}", tc),
		}
		let dig1 = format!("{}", hex_to_int(&current[0..1]) & 3);
		let dig2 = format!("{}", hex_to_int(&current[1..2]));
		let dig3 = format!("{}", hex_to_int(&current[2..3]));
		let dig4 = format!("{}", hex_to_int(&current[3..4]));
		let formatted = format!("{}{}{}{}{}", type_code, dig1, dig2, dig3, dig4);
		dtc.push(Box::leak(formatted.into_boxed_str()));
		current = &current[4..];
	}
	dtc
}

// ------------------------------------------------------------------------ //

/// class OBDPort:
/// """ OBDPort abstracts all communication with OBD-II device."""
/// def __init__(self,portnum,_notify_window,SERTIMEOUT,RECONNATTEMPTS):
/// """Initializes port by resetting device and gettings supported PIDs. """
/// # These should really be set by the user.
/// baud     = 9600
/// databits = 8
/// par      = serial.PARITY_NONE  # parity
/// sb       = 1                   # stop bits
/// to       = SERTIMEOUT
/// self.ELMver = "Unknown"
/// self.State = 1 #state SERIAL is 1 connected, 0 disconnected (connection failed)
///
/// self._notify_window=_notify_window
/// wx.PostEvent(self._notify_window, DebugEvent([1,"Opening interface (serial port)"]))
///
/// try:
/// self.port = serial.Serial(portnum,baud, \
/// parity = par, stopbits = sb, bytesize = databits,timeout = to)
///
/// except serial.SerialException:
/// self.State = 0
/// return None
///
/// wx.PostEvent(self._notify_window, DebugEvent([1,"Interface successfully " + self.port.portstr +
/// " opened"])) wx.PostEvent(self._notify_window, DebugEvent([1,"Connecting to ECU..."]))
///
/// count=0
/// while 1: #until error is returned try to connect
/// try:
/// self.send_command("atz")   # initialize
/// except serial.SerialException:
/// self.State = 0
/// return None
///
/// self.ELMver = self.get_result()
/// wx.PostEvent(self._notify_window, DebugEvent([2,"atz response:" + self.ELMver]))
/// self.send_command("ate0")  # echo off
/// wx.PostEvent(self._notify_window, DebugEvent([2,"ate0 response:" + self.get_result()]))
/// self.send_command("0100")
/// ready = self.get_result()
/// wx.PostEvent(self._notify_window, DebugEvent([2,"0100 response1:" + ready]))
/// if ready=="BUSINIT: ...OK":
/// ready=self.get_result()
/// wx.PostEvent(self._notify_window, DebugEvent([2,"0100 response2:" + ready]))
/// return None
/// else:
/// #ready=ready[-5:] #Expecting error message: BUSINIT:.ERROR (parse last 5 chars)
/// wx.PostEvent(self._notify_window, DebugEvent([2,"Connection attempt failed:" + ready]))
/// time.sleep(5)
/// if count==RECONNATTEMPTS:
/// self.close()
/// self.State = 0
/// return None
/// wx.PostEvent(self._notify_window, DebugEvent([2,"Connection attempt:" + str(count)]))
/// count=count+1
///
/// def close(self):
/// """ Resets device and closes all associated filehandles"""
///
/// if (self.port!= None) and self.State==1:
/// self.send_command("atz")
/// self.port.close()
///
/// self.port = None
/// self.ELMver = "Unknown"
///
/// def send_command(self, cmd):
/// """Internal use only: not a public interface"""
/// if self.port:
/// self.port.flushOutput()
/// self.port.flushInput()
/// for c in cmd:
/// self.port.write(c)
/// self.port.write("\r\n")
/// wx.PostEvent(self._notify_window, DebugEvent([3,"Send command:" + cmd]))
///
/// def interpret_result(self,code):
/// """Internal use only: not a public interface"""
/// # Code will be the string returned from the device.
/// # It should look something like this:
/// # '41 11 0 0\r\r'
///
/// # 9 seems to be the length of the shortest valid response
/// if len(code) < 7:
/// raise "BogusCode"
///
/// # get the first thing returned, echo should be off
/// code = string.split(code, "\r")
/// code = code[0]
///
/// #remove whitespace
/// code = string.split(code)
/// code = string.join(code, "")
///
/// #cables can behave differently
/// if code[:6] == "NODATA": # there is no such sensor
/// return "NODATA"
///
/// # first 4 characters are code from ELM
/// code = code[4:]
/// return code
///
/// def get_result(self):
/// """Internal use only: not a public interface"""
/// time.sleep(0.1)
/// if self.port:
/// buffer = ""
/// while 1:
/// c = self.port.read(1)
/// if c == '\r' and len(buffer) > 0:
/// break
/// else:
/// if buffer != "" or c != ">": #if something is in buffer, add everything
/// buffer = buffer + c
/// wx.PostEvent(self._notify_window, DebugEvent([3,"Get result:" + buffer]))
/// return buffer
/// else:
/// wx.PostEvent(self._notify_window, DebugEvent([3,"NO self.port!" + buffer]))
/// return None
///
/// # get sensor value from command
/// def get_sensor_value(self,sensor):
/// """Internal use only: not a public interface"""
/// cmd = sensor.cmd
/// self.send_command(cmd)
/// data = self.get_result()
///
/// if data:
/// data = self.interpret_result(data)
/// if data != "NODATA":
/// data = sensor.value(data)
/// else:
/// return "NORESPONSE"
/// return data
///
/// # return string of sensor name and value from sensor index
/// def sensor(self , sensor_index):
/// """Returns 3-tuple of given sensors. 3-tuple consists of
/// (Sensor Name (string), Sensor Value (string), Sensor Unit (string) ) """
/// sensor = obd_sensors.SENSORS[sensor_index]
/// r = self.get_sensor_value(sensor)
/// return (sensor.name,r, sensor.unit)
///
/// def sensor_names(self):
/// """Internal use only: not a public interface"""
/// names = []
/// for s in obd_sensors.SENSORS:
/// names.append(s.name)
/// return names
///
/// def get_tests_MIL(self):
/// statusText=["Unsupported","Supported - Completed","Unsupported","Supported - Incompleted"]
///
/// statusRes = self.sensor(1)[1] #GET values
/// statusTrans = [] #translate values to text
///
/// statusTrans.append(str(statusRes[0])) #DTCs
///
/// if statusRes[1]==0: #MIL
/// statusTrans.append("Off")
/// else:
/// statusTrans.append("On")
///
/// for i in range(2,len(statusRes)): #Tests
/// statusTrans.append(statusText[statusRes[i]])
///
/// return statusTrans
///
/// #
/// # fixme: j1979 specifies that the program should poll until the number
/// # of returned DTCs matches the number indicated by a call to PID 01
/// #
/// def get_dtc(self):
/// """Returns a list of all pending DTC codes. Each element consists of
/// a 2-tuple: (DTC code (string), Code description (string) )"""
/// dtcLetters = ["P", "C", "B", "U"]
/// r = self.sensor(1)[1] #data
/// dtcNumber = r[0]
/// mil = r[1]
/// DTCCodes = []
///
///
/// print ("Number of stored DTC:" + str(dtcNumber) + " MIL: " + str(mil))
/// # get all DTC, 3 per mesg response
/// for i in range(0, ((dtcNumber+2)/3)):
/// self.send_command(GET_DTC_COMMAND)
/// res = self.get_result()
/// print("DTC result:" + res)
/// for i in range(0, 3):
/// val1 = hex_to_int(res[3+i*6:5+i*6])
/// val2 = hex_to_int(res[6+i*6:8+i*6]) #get DTC codes from response (3 DTC each 2 bytes)
/// val  = (val1<<8)+val2 #DTC val as int
///
/// if val==0: #skip fill of last packet
/// break
///
/// DTCStr=dtcLetters[(val&0xC000)>14]+str((val&0x3000)>>12)+str(val&0x0fff)
///
/// DTCCodes.append(["Active",DTCStr])
///
/// #read mode 7
/// self.send_command(GET_FREEZE_DTC_COMMAND)
/// res = self.get_result()
///
/// if res[:7] == "NO DATA": #no freeze frame
/// return DTCCodes
///
/// print ("DTC freeze result:" + res)
/// for i in range(0, 3):
/// val1 = hex_to_int(res[3+i*6:5+i*6])
/// val2 = hex_to_int(res[6+i*6:8+i*6]) #get DTC codes from response (3 DTC each 2 bytes)
/// val  = (val1<<8)+val2 #DTC val as int
///
/// if val==0: #skip fill of last packet
/// break
///
/// DTCStr=dtcLetters[(val&0xC000)>14]+str((val&0x3000)>>12)+str(val&0x0fff)
/// DTCCodes.append(["Passive",DTCStr])
///
/// return DTCCodes
///
/// def clear_dtc(self):
/// """Clears all DTCs and freeze frame data"""
/// self.send_command(CLEAR_DTC_COMMAND)
/// r = self.get_result()
/// return r
///
/// def log(self, sensor_index, filename):
/// file = open(filename, "w")
/// start_time = time.time()
/// if file:
/// data = self.sensor(sensor_index)
/// file.write("%s     \t%s(%s)\n" % \
/// ("Time", string.strip(data[0]), data[2]))
/// while 1:
/// now = time.time()
/// data = self.sensor(sensor_index)
/// line = "%.6f,\t%s\n" % (now - start_time, data[1])
/// file.write(line)
/// file.flush()
pub const MOMO: u8 = 0;
