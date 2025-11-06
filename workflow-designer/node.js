function registerNode(type,label,properties,onClick){

    function MyNode(){
        this.addInput("in", "number");
        this.addOutput("out", "number");
        this.properties = properties;
        this.onSelected =()=>{
             onClick(this);
        }
    }
    LiteGraph.registerNodeType(type, MyNode);
    MyNode.title = label;
    MyNode.desc = label;
}

(function (global) {
    var LiteGraph = global.LiteGraph;

    registerNode("core/hello","Hello",{message : ""},(node)=>{ console.log(node)});

    function Start() {
        this.addOutput("next", "number");
        
        this.addWidget("button", "Add Input", "condition", () => {
            var name = "input " + (this.condition++);
            this.addWidget("text", name, name, () => { });
        });

        this.properties = { schedule: "" ,inputs :{}};
        this.onSelected =()=>{
            this.mynewPorp = 10;
            console.log(this.properties)
        }

    }

    Start.title = "Start";
    Start.desc = "Start";

    Start.prototype.onExecute = function () { return {}
    };

    LiteGraph.registerNodeType("core/Start", Start);
    //-------------------------------------

    function End() {
        this.addInput("id", "number");

    }

    End.title = "End";
    End.desc = "End";

    End.prototype.onExecute = function () { };
    LiteGraph.registerNodeType("core/End", End);

    //-------------------------------------
    function Condition() {
        this.condition = 1;
        this.addInput("id", "number");
        this.addWidget("button", "Add Condition", "condition", () => {
            var name = "condition " + (this.condition++);
            this.addWidget("text", name, name, () => { });
            this.addOutput(name, "number");
        });
    }

    Condition.title = "Condition";
    Condition.desc = "Condition";

    Condition.prototype.onExecute = function () { };

    LiteGraph.registerNodeType("core/Condition", Condition);

    //-------------------------------------
    function Log() {
        this.Log = 1;
        this.addInput("id", "number");
        this.addWidget("text", "->message", "message", () => { });
        this.addOutput("next", "number");
    }

    Log.title = "Log";
    Log.desc = "Log";

    Log.prototype.onExecute = function () { };

    LiteGraph.registerNodeType("core/Log", Log);

    //-------------------------------------
    function ReadFile() {
        this.ReadFile = 1;
        this.addInput("id", "number");
        this.addWidget("text", "-> path", "path", () => { });
        this.addWidget("text", "content ->", "content", () => { });
        this.addWidget("text", "error ->", "error", () => { });
        this.addOutput("next", "number");
    }

    ReadFile.title = "ReadFile";
    ReadFile.desc = "ReadFile";

    ReadFile.prototype.onExecute = function () { };

    LiteGraph.registerNodeType("file/ReadFile", ReadFile);
    //-------------------------------------
    function WriteFile() {
        this.WriteFile = 1;
        this.addInput("id", "number");
        this.addWidget("text", "-> path", "path", () => { });
        this.addWidget("text", "-> content", "content", () => { });
        this.addWidget("text", "status ->", "status", () => { });
        this.addWidget("text", "error ->", "error", () => { });
        this.addOutput("next", "number");
    }

    WriteFile.title = "WriteFile";
    WriteFile.desc = "WriteFile";

    WriteFile.prototype.onExecute = function () { };

    LiteGraph.registerNodeType("file/WriteFile", WriteFile);
    //-------------------------------------
    function AppendFile() {
        this.AppendFile = 1;
        this.addInput("id", "number");
        this.addWidget("text", "-> path", "path", () => { });
        this.addWidget("text", "-> content", "content", () => { });
        this.addWidget("text", "status ->", "status", () => { });
        this.addWidget("text", "error ->", "error", () => { });
        this.addOutput("next", "number");
    }

    AppendFile.title = "AppendFile";
    AppendFile.desc = "AppendFile";

    AppendFile.prototype.onExecute = function () { };

    LiteGraph.registerNodeType("file/AppendFile", AppendFile);
    //-------------------------------------
    function DeleteFile() {
        this.DeleteFile = 1;
        this.addInput("id", "number");
        this.addWidget("text", "-> path", "path", () => { });
        this.addWidget("text", "status ->", "status", () => { });
        this.addWidget("text", "error ->", "error", () => { });
        this.addOutput("next", "number");
    }

    DeleteFile.title = "DeleteFile";
    DeleteFile.desc = "DeleteFile";

    DeleteFile.prototype.onExecute = function () { };

    LiteGraph.registerNodeType("file/DeleteFile", DeleteFile);
   

})(this);