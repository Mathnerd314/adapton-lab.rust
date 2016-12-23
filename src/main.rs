#![feature(field_init_shorthand)]
#![feature(rustc_private)]
//#![feature(custom_derive)]

use std::fmt::Debug;
//use std::hash::Hash;
use std::rc::Rc;
//use std::path::Path;

extern crate serialize;
extern crate time;
extern crate csv;
extern crate rand;

#[macro_use]
extern crate adapton;

//use adapton::macros::*;
use adapton::collections::*;
use adapton::engine::*;
use rand::{Rng, SeedableRng};
use std::marker::PhantomData;

// --- Todo: Move to labdef:

/// A bit that controls how names are placed in the input; See `README.md` for more.
#[derive(Clone,Debug)]
pub enum NominalStrategy {
  Regular,
  ByContent,
}

/// Parameters for generating and editing input; See `README.md` for more.
#[derive(Clone,Debug)]
pub struct GenerateParams {
  pub size: usize, 
  pub gauge: usize, 
  pub nominal_strategy:NominalStrategy
}

/// Generic method for generating a random input.
/// See `README.md` for more.
pub trait Generate<T> {
  fn generate<R:Rng>(rng:&mut R, params:&GenerateParams) -> T;
} 

/// Generic process for editing an input randomly, in a stateful sequence of edits.
/// See `README.md` for more.
pub trait Edit<T,S> : Clone {
  fn edit_init<R:Rng>(rng:&mut R, params:&GenerateParams) -> S;
  fn edit<R:Rng>(pre_edit:T, edit_state:S, rng:&mut R, params:&GenerateParams) -> (T, S);
}

/// Generic notion of a computation to run naively and incrementally.
/// It has specific `Input` and `Output` types, and a way to `compute`
/// the `Output` from the `Input`.
/// See `README.md` for more.
pub trait Compute<Input,Output> {
  fn compute(Input) -> Output;
}

/// Generic notion of an Incremental Computation to Evaluate and Test.
/// We instantiate this structure once for each test in our test suite.
/// We implement the `LabExp` trait generically for this structure.
pub struct TestComputer<Input,EditSt,Output,
                        InputDist:Generate<Input>+Edit<Input,EditSt>,
                        Computer:Compute<Input,Output>> {
  identity:  Name,
  computer:  PhantomData<Computer>,
  input:     PhantomData<Input>,
  editst:    PhantomData<EditSt>,
  inputdist: PhantomData<InputDist>,
  output:    PhantomData<Output>
}

/// Parameters to running a single lab experiment.
#[derive(Clone,Debug)]
pub struct LabExpParams {
  pub sample_params: SampleParams,
  // TODO: Pretty-print input and output structures; graphmovie dump of experiment
  /// Number of change-batches to perform in a loop; each is interposed with computing the new output.
  pub change_batch_loopc: usize,
}

/// Parameters for collecting a single sample.  In addition to these
/// parameters, the experiment maintains a Rng based on the
/// input_seeds, below; this Rng is given to Edit::edit to generate
/// psuedo-random edits, in batches.  For each engine, this Rng is
/// sequenced across successive samples.  Given an input_seeds vector,
/// there is one unique Rng sequence for each engine's sequence of
/// samples.
#[derive(Clone,Debug)]
pub struct SampleParams {
  /// We convert this seed into a random-number-generator before generating and editing.
  pub input_seeds:       Vec<usize>, 
  /// Other parameters for generating the input.
  pub generate_params:   GenerateParams, 
  /// Whether to validate the output after each computation using the naive and DCG engines
  pub validate_output:   bool,
  /// Size of each batch of changes.
  pub change_batch_size: usize,
}

/// The result of a lab is a sequence of samples.
#[derive(Clone,Debug)]
pub struct LabExpResults {
  pub samples: Vec<Sample>
}

/// The experiment consists of a loop over samples.  For each sample,
/// we switch back and forth between using the Naive engine, and using
/// the DCG engine.  We want to interleave this way for each sample in
/// order to compare outputs and metrics (counts and timings) on a
/// fine-grained scale.
#[derive(Clone,Debug)]
pub struct Sample {
  pub params:       SampleParams,
  pub batch_name:   usize,   // Index/name the change batches; one sample per compute + change batch
  pub dcg_sample:   EngineSample,
  pub naive_sample: EngineSample,
  pub output_valid: Option<bool>
}

/// To sample a single engine, we record metrics for processing the
/// input (left vertical edge in `README.md` diagram).
#[derive(Clone,Debug)]
pub struct EngineSample {
  pub process_input:    EngineMetrics,
  pub compute_output:   EngineMetrics,
}

/// For each engine, for each sampled subcomputation, we record the
/// real time (in nanoseconds) and engine-based counters for DCG costs.
#[derive(Clone,Debug)]
pub struct EngineMetrics {
  pub time_ns:    u64,
  pub engine_cnt: Cnt,
}

// --- Todo: Move to labrun:

pub trait SampleGen {
  fn sample(self:&mut Self) -> Option<Sample>;
}

pub struct TestEngineState<Input,EditSt,Output,
                           InputDist:Generate<Input>+Edit<Input,EditSt>,
                           Computer:Compute<Input,Output>> {
  pub engine:   Engine,
  pub input:    Option<(Input,EditSt)>,
  inputdist:    PhantomData<InputDist>,
  computer:     PhantomData<Computer>,
  output:       PhantomData<Output>,
}

pub struct TestState<R:Rng+Clone,
                     Input,EditSt,Output,
                     InputDist:Generate<Input>+Edit<Input,EditSt>,
                     Computer:Compute<Input,Output>> {
  pub params:           LabExpParams,
  pub rng:              Box<R>,
  pub change_batch_num: usize,
  pub dcg_state:   TestEngineState<Input,EditSt,Output,InputDist,Computer>,
  pub naive_state: TestEngineState<Input,EditSt,Output,InputDist,Computer>,
  pub samples:     Vec<Sample>,
}

      
fn get_engine_metrics<X,F:FnOnce() -> X> (thunk:F) -> (X,EngineMetrics)
{
  let time_start = time::precise_time_ns();
  let (x,cnt) = cnt(thunk);
  let time_end = time::precise_time_ns();
  return (x, EngineMetrics{
    time_ns:time_end - time_start,
    engine_cnt:cnt,
  })
}

fn get_engine_sample
  <R:Rng+Clone,
   Input:Clone+Debug,
   EditSt,Output:Debug,   
   InputDist:Generate<Input>+Edit<Input,EditSt>,
   Computer:Compute<Input,Output>
   > 
  (rng:&mut R, params:&SampleParams, input:Option<(Input,EditSt)>) -> (Output,Input,EditSt,EngineSample) 
{
  let mut rng2 = rng;
  
  let ((edited_input, editst), process_input) : ((Input,EditSt),EngineMetrics) = 
    match input {
      None => 
        get_engine_metrics(
          move || ( InputDist::generate(&mut rng2, &params.generate_params), 
                    InputDist::edit_init(&mut rng2, &params.generate_params ))),
      Some((input, editst)) => 
        get_engine_metrics(
          move || InputDist::edit(input, editst, &mut rng2, &params.generate_params))
    };
  println!("EngineSample::process_input: {:?}", process_input); // XXX Temp  

  let input2 = edited_input.clone();
  let (output, compute_output): (Output,EngineMetrics) 
    = ns(name_of_str("compute"),
         move || get_engine_metrics(move || Computer::compute(input2) ));

  println!("EngineSample::compute_output: {:?}", compute_output); // XXX Temp  

  if false {
    println!(" Input: {:?}", edited_input); // XXX Temp
    println!("Output: {:?}", output); // XXX Temp
  };
  
  let engine_sample = EngineSample{
    process_input,
    compute_output,
  };
  //println!("{:?}", engine_sample); // XXX Temp
  return (output, edited_input, editst, engine_sample)
}

fn get_sample_gen
  <Input:Clone+Debug,
   EditSt,
   Output:Eq+Debug,
   InputDist:Generate<Input>+Edit<Input,EditSt>,
   Computer:Compute<Input,Output>> 
  (params:&LabExpParams) 
   -> TestState<rand::StdRng,Input,EditSt,Output,InputDist,Computer> 
{
  // Create empty DCG; TODO-Minor-- Make the API for this better.
  let _ = init_dcg(); assert!(engine_is_dcg());
  let empty_dcg = use_engine(Engine::Naive); // TODO-Minor: Rename this operation: "engine_swap" or something 
  let rng = SeedableRng::from_seed(params.sample_params.input_seeds.as_slice());
  //let editst_init = InputDist::edit_init(&mut rng, & params.sample_params.generate_params);
  TestState{
    params:params.clone(),
    rng:Box::new(rng),
    dcg_state:TestEngineState{
      input:  None,
      engine: empty_dcg, // empty DCG      
      output: PhantomData, inputdist: PhantomData, computer: PhantomData,      
    },
    naive_state:TestEngineState{
      input:  None,
      engine: Engine::Naive, // A constant
      output: PhantomData, inputdist: PhantomData, computer: PhantomData,
    },
    change_batch_num: 0,
    samples:vec![],
  }
}

/// Advances the TestState forward by one sample of each engine.  For
/// each engine, we process the current input (either generating it,
/// or editing it) and we compute a new output over this processed input.
/// Optionally, we compare the outputs of the engines for equality.
impl<Input:Clone+Debug,EditSt,Output:Eq+Debug,
     InputDist:Generate<Input>+Edit<Input,EditSt>,
     Computer:Compute<Input,Output>>
  SampleGen for TestState<rand::StdRng,Input,EditSt,Output,InputDist,Computer> {
    fn sample (self:&mut Self) -> Option<Sample> {
      if self.change_batch_num > self.params.change_batch_loopc {
        None 
      } else { // Collect the next sample, for each engine, using get_engine_sample.
        let mut dcg_state = TestEngineState{ input: None, engine: Engine::Naive, 
                                             output: PhantomData, inputdist: PhantomData, computer: PhantomData };
        std::mem::swap(&mut dcg_state, &mut self.dcg_state );
        let mut naive_state = TestEngineState{ input: None, engine: Engine::Naive, 
                                               output: PhantomData, inputdist: PhantomData, computer: PhantomData };
        std::mem::swap(&mut naive_state, &mut self.naive_state );

        // Run Naive Version
        println!("Naive - - - - - ");
        let _ = use_engine(Engine::Naive); assert!(engine_is_naive());
        let mut rng = self.rng.clone(); // Restore Rng
        let (naive_output, naive_input_edited, naive_editst, naive_sample) = 
          get_engine_sample::<rand::StdRng,Input,EditSt,Output,InputDist,Computer>
          (&mut rng, &self.params.sample_params, naive_state.input);
        self.naive_state.input = Some((naive_input_edited, naive_editst)); // Save the input and input-editing state

        // Run DCG Version
        println!("DCG - - - - - ");
        let _ = use_engine(dcg_state.engine); // Restore saved DCG
        assert!(engine_is_dcg()); // This really is the DCG version
        let mut rng = self.rng.clone(); // Restore Rng
        let (dcg_output, dcg_input_edited, dcg_editst, dcg_sample) = 
          get_engine_sample::<rand::StdRng,Input,EditSt,Output,InputDist,Computer>
          (&mut rng, &self.params.sample_params, dcg_state.input);
        self.dcg_state.engine = use_engine(Engine::Naive); // Swap out the DCG
        self.dcg_state.input = Some((dcg_input_edited, dcg_editst)); // Save the input and input-editing state
        
        // Save the Rng for the next sample.
        self.rng = Box::new(*rng);

        // Compare the two outputs for equality
        let output_valid = if self.params.sample_params.validate_output { 
          Some ( dcg_output == naive_output )
        } else { None } ;

        let sample = Sample{
          params:self.params.sample_params.clone(),
          batch_name:self.change_batch_num,
          dcg_sample,
          naive_sample,
          output_valid,
        };
        self.change_batch_num += 1;
        Some(sample)
      }
    }
  }

/// Lab experiment: Hides the Input, Output and Compute types of a
/// TestComputer, abstracting over them.
pub trait LabExp {
  fn name(self:&Self) -> Name;
  fn run(self:&Self, params:&LabExpParams) -> LabExpResults;
}

/// Lab experiment implementation: Implements the LabExp trait for any
/// TestComputer instantiation.
impl<Input:Clone+Debug,EditSt,Output:Eq+Debug,
     InputDist:'static+Generate<Input>+Edit<Input,EditSt>,
     Computer:'static+Compute<Input,Output>>
  LabExp for TestComputer<Input,EditSt,Output,InputDist,Computer> {
    fn name(self:&Self) -> Name { self.identity.clone() }
    fn run(self:&Self, params:&LabExpParams) -> LabExpResults 
    {            
      let mut st = get_sample_gen::<Input,EditSt,Output,InputDist,Computer>(params);
      loop {
        println!("{:?}", self.name());
        let sample = (&mut st).sample();
        //println!("{:?}", sample);
        match sample {
          Some(_) => continue,
          None => break,
        }
      };
      return LabExpResults {
        samples: st.samples,
      }
    }
  }


// -- Todo: Keep in main.rs:

  

// fn csv_of_runtimes(path:&str, samples: Vec<Sample>) {
//   let path = Path::new(path);
//   let mut writer = csv::Writer::from_file(path).unwrap();
//   for r in samples.into_iter() {
//     //println!("{:?}",r);
//     //writer.encode(r).ok().expect("CSV writer error");
//   }
// }

fn labexp_params_defaults() -> LabExpParams {
  return LabExpParams {
    sample_params: SampleParams{
      input_seeds: vec![0],
      generate_params: GenerateParams{
        size:10,
        gauge:1,
        nominal_strategy:NominalStrategy::Regular,
      },
      validate_output: true,
      change_batch_size: 1,
    },
    change_batch_loopc:10,
  }
}


// TODO -- Put these implementations into a 'catalog' module.

#[derive(Clone,Debug)]
pub struct UniformPrepend<T,S> { t:PhantomData<T>, s:PhantomData<S> }

impl<S> Generate<List<usize>> for UniformPrepend<List<usize>,S> {
  fn generate<R:Rng>(rng:&mut R, params:&GenerateParams) -> List<usize> {
    let mut l : List<usize> = list_nil();
    for i in 0..params.size {
      if i % params.gauge == 0 {
        l = list_art(cell(name_of_usize(i), l));
        l = list_name(name_of_usize(i), l);
      } else { } ;
      let elm : usize = rng.gen() ;
      let elm = elm % 1000 ;
      l = list_cons(elm,  l);
    } ;
    l
  }
}

impl Edit<List<usize>, usize> for UniformPrepend<List<usize>,usize> {
  fn edit_init<R:Rng>(_rng:&mut R, params:&GenerateParams) -> usize { 
    return params.size // Initial editing state = The size of the generated input
  }
  fn edit<R:Rng>(l_preedit:List<usize>, 
                 next_name:usize,
                 rng:&mut R, params:&GenerateParams) -> (List<usize>, usize) {
    let mut l = l_preedit ;
    let i = next_name ;
    if i % params.gauge == 0 {
      l = list_art(cell(name_of_usize(i), l));
      l = list_name(name_of_usize(i), l);      
    } else { } ;
    let elm : usize = rng.gen() ;
    let elm = elm % 1000 ;
    (list_cons(elm, l), i + 1)
  }
}


//#[derive(Clone,Debug)]
//pub struct UniformPrepend<T,S> { T:PhantomData<T>, S:PhantomData<S> }

type Pt2D = (usize,usize); // TODO Fix this

impl<S> Generate<List<Pt2D>> for UniformPrepend<List<Pt2D>,S> { // TODO
  fn generate<R:Rng>(_rng:&mut R, _params:&GenerateParams) -> List<Pt2D> {
    //panic!("TODO")
    list_nil()
  }
}

impl Edit<List<Pt2D>,usize> for UniformPrepend<List<Pt2D>,usize> { // TODO
  fn edit_init<R:Rng>(_rng:&mut R, _params:&GenerateParams) -> usize { 0 }
  fn edit<R:Rng>(state:List<Pt2D>, st:usize, _rng:&mut R, _params:&GenerateParams) -> (List<Pt2D>, usize) {
    //TODO
    (state, st)
  }
}


#[derive(Clone,Debug)]
pub struct LazyMap { }
#[derive(Clone,Debug)]
pub struct EagerMap { }
#[derive(Clone,Debug)]
pub struct LazyFilter { }
#[derive(Clone,Debug)]
pub struct EagerFilter { }
#[derive(Clone,Debug)]
pub struct Reverse { }
#[derive(Clone,Debug)]
pub struct LazyMergesort { }
#[derive(Clone,Debug)]
pub struct EagerMergesort { }
#[derive(Clone,Debug)]
pub struct Quickhull { }

impl Compute<List<usize>,List<usize>> for EagerMap {
  fn compute(inp:List<usize>) -> List<usize> {
    list_map_eager(inp,Rc::new(|x| x * x))
  }
}

impl Compute<List<usize>,List<usize>> for EagerFilter {
  fn compute(inp:List<usize>) -> List<usize> {
    list_filter_eager(inp,Rc::new(|x:&usize| (*x) % 3 == 0))
  }
}

impl Compute<List<usize>,List<usize>> for LazyMap {
  fn compute(inp:List<usize>) -> List<usize> {
    list_map_lazy(inp,Rc::new(|x| x * x))
  }
}

impl Compute<List<usize>,List<usize>> for LazyFilter {
  fn compute(inp:List<usize>) -> List<usize> {
    list_filter_lazy(inp,Rc::new(|x:&usize| (*x) % 3 == 0))
  }
}

impl Compute<List<usize>,List<usize>> for Reverse {
  fn compute(inp:List<usize>) -> List<usize> {
    list_reverse(inp, list_nil())
  }
}

impl Compute<List<usize>,List<usize>> for LazyMergesort {
  fn compute(inp:List<usize>) -> List<usize> {    
    let tree = ns( name_of_str("tree_of_list"), 
                   move ||tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    // TODO: Is this eager or lazy?
    mergesort_list_of_tree2(tree,None)
  }
}

impl Compute<List<usize>,List<usize>> for EagerMergesort {
  fn compute(inp:List<usize>) -> List<usize> {
    let tree = ns( name_of_str("tree_of_list"), 
                   move ||tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    // TODO: Is this eager or lazy?
    mergesort_list_of_tree2(tree,None)
  }
}

impl Compute<List<Pt2D>,List<Pt2D>> for Quickhull {
  fn compute(inp:List<Pt2D>) -> List<Pt2D> {
    //panic!("TODO")
    inp
  }
}

// TODO: Move the start of this file (above this marker) to separate modules/files: 
//
// -- The labdef module abstractly defines lab experiments:
// parameters, samples and generic traits.
//
// -- The labrun module implements the LabExp trait for any
// TestComputer instantiation.
//
// -- The catalog module allows us to instantiate TestComputer in
// standard ways, exercising the Adapton collections library.
//

#[macro_export]
macro_rules! testcomputer {
  ( $name:expr, $inp:ty, $editst:ty, $out:ty, $dist:ty, $comp:ty ) => {{ 
    Box::new( 
      TestComputer
        ::<$inp,$editst,$out,$dist,$comp>
      { 
        identity:$name,
        input:PhantomData,
        editst:PhantomData,
        output:PhantomData,
        inputdist:PhantomData,
        computer:PhantomData
      }) 
  }}
}


/// This is the master list of all tests in the current Adapton Lab
pub fn all_tests() -> Vec<Box<LabExp>> {
  return vec![
    testcomputer!(name_of_str("list-eager-map"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerMap)
      ,
    testcomputer!(name_of_str("list-eager-filter"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerFilter)
      ,
    testcomputer!(name_of_str("list-lazy-map"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyMap)
      ,
    testcomputer!(name_of_str("list-lazy-filter"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyFilter)
      ,
    testcomputer!(name_of_str("list-reverse"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  Reverse)
      ,
    testcomputer!(name_of_str("eager-mergesort"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerMergesort)
      ,
    testcomputer!(name_of_str("lazy-mergesort"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyMergesort)
      ,
    testcomputer!(name_of_str("list-quickhull"),
                  List<Pt2D>, usize,
                  List<Pt2D>,
                  UniformPrepend<_,_>,
                  Quickhull)
      ,
  ]
}



fn run_all_tests() {
  let params = labexp_params_defaults();
  let tests = all_tests();
  for test in tests.iter() {
    println!("Test: {:?}", test.name());
    let results = test.run(&params);
    drop(results)
  }
}

#[test]
fn test_all() { run_all_tests() }
fn main2() { run_all_tests() }

fn main () {
  use std::thread;
  let child =
    thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(move || { 
      main2()
    });
  let _ = child.unwrap().join();
}
