﻿using BenchmarkDotNet.Configs;
using BenchmarkDotNet.Running;
using BenchmarkDotNet.Validators;
using NHapi.Base.Parser;
using NHapi.Base.Util;
using System;
using System.Linq;

namespace ConsoleApp1
{
    internal class Program
    {
        private static void Main(string[] args)
        {
            // //RUST
            using (var mh = Native.BuildMessage(NhapiVsRustHL7.ORU_TEXT))
            { //pointer to Message

                using (var fieldValue = Native.GetField(mh.DangerousGetHandle(), "OBR", 7))
                {
                    var fieldValueAsString = fieldValue.AsString();
                    Console.WriteLine($"Rust retrieved value: '{fieldValueAsString}'");
                } //dispose of string handle, freeing up string memeory on the rust side.
            }


            using (var fieldValue = Native.GetFieldFromMessage(NhapiVsRustHL7.ORU_TEXT, "OBR", 7))
            {
                var fieldValueAsString = fieldValue.AsString();
                Console.WriteLine($"Rust non-parser retrieved value: '{fieldValueAsString}'");
            } //dispose of string handle, freeing up string memeory on the rust side.




            //HL7-DotNetCore
            var hl7Message = new HL7.Dotnetcore.Message(NhapiVsRustHL7.ORU_TEXT);
            hl7Message.ParseMessage();
            var v = hl7Message.GetValue("OBR.7"); //get a rando field from the middle of the thing
            Console.WriteLine($"HL7-DotNetCore retrieved value: '{v}'");

            //NHAPI
            var parser = new PipeParser();
            var hl7Message2 = parser.Parse(NhapiVsRustHL7.ORU_TEXT) as NHapi.Model.V24.Message.ORU_R01;
            var t = new Terser(hl7Message2);
            var field = t.Get("/.OBR-7"); //get a rando field from the middle of the thing
            Console.WriteLine($"NHapi retrieved value: '{field}'");

            //Tims Simple Parser
            var timsParser = new TimsSimpleParser();
            var hl7Message3 = timsParser.Parse(NhapiVsRustHL7.ORU_TEXT);
            v = hl7Message3.GetField("OBR", 7);
            Console.WriteLine($"Tims Simple .Net Parser retrieved value: '{v}'");

            v = TimsSimpleParser.GetFieldFromMessage(NhapiVsRustHL7.ORU_TEXT, "OBR", 7);
            Console.WriteLine($"Tims Simple not-a-parser retrieved value: '{v}'");

            /* for (var i = 0; i < 100_000; i++)
             {
                 var timsParser = new TimsSimpleParser();
                 var hl7Message3 = timsParser.Parse("MSH|^~\\&|GHH LAB|ELAB-3|GHH OE|BLDG4|200202150930||ORU^R01|CNTRL-3456|P|2.4\rPID|||555-44-4444||EVERYWOMAN^EVE^E^^^^L|JONES|19620320|F|||153 FERNWOOD DR.^^STATESVILLE^OH^35292||(206)3345232|(206)752-121||||AC555444444||67-A4335^OH^20030520\rOBR|1|845439^GHH OE|1045813^GHH LAB|15545^GLUCOSE|||200202150730|||||||||555-55-5555^PRIMARY^PATRICIA P^^^^MD^^|||||||||F||||||444-44-4444^HIPPOCRATES^HOWARD H^^^^MD\rOBX|1|SN|1554-5^GLUCOSE^POST 12H CFST:MCNC:PT:SER/PLAS:QN||^182|mg/dl|70_105|H|||F\r");
                 var v = hl7Message3.GetField("OBR", 7);
             }*/



            //Console.Read();
            var summary = BenchmarkRunner.Run<NhapiVsRustHL7>(new AllowNonOptimized()); //HL7-DotNet has published a debug build :(

        }

        public class AllowNonOptimized : ManualConfig
        {
            public AllowNonOptimized()
            {
                Add(JitOptimizationsValidator.DontFailOnError); // ALLOW NON-OPTIMIZED DLLS

                Add(DefaultConfig.Instance.GetLoggers().ToArray()); // manual config has no loggers by default
                Add(DefaultConfig.Instance.GetExporters().ToArray()); // manual config has no exporters by default
                Add(DefaultConfig.Instance.GetColumnProviders().ToArray()); // manual config has no columns by default
            }
        }
    }
}