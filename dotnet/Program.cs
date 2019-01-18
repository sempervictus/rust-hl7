﻿using BenchmarkDotNet.Running;
using NHapi.Base.Parser;
using NHapi.Base.Util;
using System.Linq;

namespace ConsoleApp1
{
    internal class Program
    {
        private static void Main(string[] args)
        {
            var _parser = new PipeParser();
            var hl7Message = _parser.Parse(ORU_TEXT)as NHapi.Model.V24.Message.ORU_R01;
            var field = hl7Message.PATIENT_RESULTs.First().ORDER_OBSERVATIONs.First().OBR.GetOrderingProvider(0);

            var summary = BenchmarkRunner.Run<nhapi>();
        }


        const string ORU_TEXT = @"MSH|^~\&|GHH LAB|ELAB-3|GHH OE|BLDG4|200202150930||ORU^R01|CNTRL-3456|P|2.4
 PID|||555-44-4444||EVERYWOMAN^EVE^E^^^^L|JONES|19620320|F|||153 FERNWOOD DR.^^STATESVILLE^OH^35292||(206)3345232|(206)752-121||||AC555444444||67-A4335^OH^20030520
 OBR|1|845439^GHH OE|1045813^GHH LAB|15545^GLUCOSE|||200202150730|||||||||555-55-5555^PRIMARY^PATRICIA P^^^^MD^^|||||||||F||||||444-44-4444^HIPPOCRATES^HOWARD H^^^^MD
 OBX|1|SN|1554-5^GLUCOSE^POST 12H CFST:MCNC:PT:SER/PLAS:QN||^182|mg/dl|70_105|H|||F";
    }
}