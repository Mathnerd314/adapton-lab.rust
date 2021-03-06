use labdef::*;
use adapton::collections::*;
use adapton::engine::*;
use rand::{Rng};
use std::marker::PhantomData;
use std::rc::Rc;
use pmfp_collections::gauged_raz::{Raz,RazTree};
use pmfp_collections::level_tree::{gen_branch_level};


#[derive(Clone,Debug)]
pub struct UniformInsert<T,S> { t:PhantomData<T>, s:PhantomData<S> }

impl<S> Generate<RazTree<usize>> for UniformInsert<RazTree<usize>, S> {
  fn generate<R:Rng> (mut rng:&mut R, params:&GenerateParams) -> RazTree<usize> {
    let mut r = Raz::new();
    for i in 0..params.size {
      if i % params.gauge == 0 {
        let l = gen_branch_level(&mut rng);
        r.archive_left( l );
      } else { } ;
      r.push_left(i);
    }
    r.unfocus()
  }
}

impl Edit<RazTree<usize>, usize> for UniformInsert<RazTree<usize>, usize> {
  fn edit_init<R:Rng>(_rng:&mut R, params:&GenerateParams) -> usize { 
    return params.size // Initial editing state = The size of the generated input
  }
  fn edit<R:Rng>(tree:RazTree<usize>, i:usize,
                 rng:&mut R, _params:&GenerateParams) -> (RazTree<usize>, usize) {
    let t = tree;
    let pos = rng.gen::<usize>() % ( i + 1 );
    let mut r = t.focus( pos ).unwrap();
    r.push_left( rng.gen() );
    let t = r.unfocus();    
    (t, i + 1)
  }
}



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
      let elm = elm % ( params.size * 100 ) ;
      l = list_cons(elm,  l);
      if i % params.gauge == 0 {
        //l = list_art(cell(name_of_usize(i), l));
        //l = list_name(name_of_usize(i), l);
      } else { } ;
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
    let elm = elm % ( params.size * 100 ) ;
    l = list_cons(elm, l);
    if i % params.gauge == 0 {
      //l = list_art(cell(name_of_usize(i), l));
      //l = list_name(name_of_usize(i), l);      
    } else { } ;
    (l, i + 1)
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
pub struct ListTree { }
#[derive(Clone,Debug)]
pub struct ListTreeMax { }
#[derive(Clone,Debug)]
pub struct ListTreeSum { }

#[derive(Clone,Debug)]
pub struct ListReverse { }

#[derive(Clone,Debug)]
pub struct LazyMergesort3 { }
#[derive(Clone,Debug)]
pub struct EagerMergesort3 { }

#[derive(Clone,Debug)]
pub struct LazyMergesort2 { }
#[derive(Clone,Debug)]
pub struct EagerMergesort2 { }

#[derive(Clone,Debug)]
pub struct LazyMergesort1 { }
#[derive(Clone,Debug)]
pub struct EagerMergesort1 { }

#[derive(Clone,Debug)]
pub struct Quickhull { }

pub struct RazTest1 {} 

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

impl ComputeDemand<List<usize>,List<usize>> for LazyMap {
  fn compute(inp:List<usize>, demand:usize) -> List<usize> {
    let out : List<usize> = list_map_lazy(inp,Rc::new(|x| x * x));
    drop( list_demand( out.clone(), demand ) );
    out
  }
}

impl ComputeDemand<List<usize>,List<usize>> for LazyFilter {
  fn compute(inp:List<usize>, demand:usize) -> List<usize> {
    let out : List<usize> = 
      list_filter_lazy(inp,Rc::new(|x:&usize| (*x) % 3 == 0));
    drop( list_demand( out.clone(), demand) );
    out
  }
}

impl Compute<List<usize>,List<usize>> for ListReverse {
  fn compute(inp:List<usize>) -> List<usize> {
    list_reverse(inp, list_nil())
  }
}

impl Compute<List<usize>,Tree<usize>> for ListTree {
  fn compute(inp:List<usize>) -> Tree<usize> {
    tree_of_list(Dir2::Left,inp)
  }
}

impl Compute<List<usize>,usize> for ListTreeMax {
  fn compute(inp:List<usize>) -> usize {
    let tree : Tree<usize> = 
      ns(name_of_str("tree_of_list"),
         move|| tree_of_list(Dir2::Left,inp));
    monoid_of_tree(tree, 0, 
                   Rc::new(|x,y| if x > y { x } else { y }))
  }
}

impl Compute<List<usize>,usize> for ListTreeSum {
  fn compute(inp:List<usize>) -> usize {
    let tree : Tree<usize> = 
      ns(name_of_str("tree_of_list"),
         move|| tree_of_list(Dir2::Left,inp));
    monoid_of_tree(tree, 0, 
                   Rc::new(|x,y| x + y ))
  }
}

impl ComputeDemand<List<usize>,List<usize>> for LazyMergesort3 {
  fn compute(inp:List<usize>, demand:usize) -> List<usize> {    
    let tree = ns( name_of_str("tree_of_list"), 
                   move ||tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let out : List<usize> = mergesort_list_of_tree2(tree,None);
    drop( list_demand( out.clone(), demand ) );
    out
  }
}

impl Compute<List<usize>,List<usize>> for EagerMergesort3 {
  fn compute(inp:List<usize>) -> List<usize> {
    let tree = 
      ns( name_of_str("tree_of_list"), 
          move || tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let sorted : List<_> = 
      ns( name_of_str("mergesort"),
          move || mergesort_list_of_tree3(tree, None));
    let sorted2 = sorted.clone();
    let tree2 = // Demand the output of mergesort (making it "eager")
      ns ( name_of_str("tree_of_list2"),
           move || tree_of_list::<_,_,Tree<_>,List<_>>(Dir2::Left,sorted) );
    // ns ( name_of_str("list_of_tree"),
    //      move || list_of_tree(tree2, Dir2::Left ) )
    drop(tree2);
    sorted2
  }
}

impl ComputeDemand<List<usize>,List<usize>> for LazyMergesort2 {
  fn compute(inp:List<usize>, demand:usize) -> List<usize> {    
    let tree = ns( name_of_str("tree_of_list"), 
                   move ||tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let out : List<usize> = mergesort_list_of_tree2(tree,None);
    drop( list_demand( out.clone(), demand) );
    out
  }
}

impl Compute<List<usize>,List<usize>> for EagerMergesort2 {
  fn compute(inp:List<usize>) -> List<usize> {
    let tree = 
      ns( name_of_str("tree_of_list"), 
          move || tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let sorted : List<_> = 
      ns( name_of_str("mergesort"),
          move || mergesort_list_of_tree2(tree, None));
    let sorted2 = sorted.clone();
    let tree2 = // Demand the output of mergesort (making it "eager")
      ns ( name_of_str("tree_of_list2"),
           move || tree_of_list::<_,_,Tree<_>,List<_>>(Dir2::Left,sorted) );
    // ns ( name_of_str("list_of_tree"),
    //      move || list_of_tree(tree2, Dir2::Left ) )
    drop(tree2);
    sorted2
  }
}

impl ComputeDemand<List<usize>,List<usize>> for LazyMergesort1 {
  fn compute(inp:List<usize>, demand:usize) -> List<usize> {    
    let tree = ns( name_of_str("tree_of_list"), 
                   move ||tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let out : List<usize> = mergesort_list_of_tree(tree);
    drop( list_demand( out.clone(), demand) );
    out
  }
}

impl Compute<List<usize>,List<usize>> for EagerMergesort1 {
  fn compute(inp:List<usize>) -> List<usize> {
    let tree = 
      ns( name_of_str("tree_of_list"), 
          move || tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let sorted : List<_> = 
      ns( name_of_str("mergesort"),
          move || mergesort_list_of_tree(tree));
    let sorted2 = sorted.clone();
    let tree2 = // Demand the output of mergesort (making it "eager")
      ns ( name_of_str("tree_of_list2"),
           move || tree_of_list::<_,_,Tree<_>,List<_>>(Dir2::Left,sorted) );
    // ns ( name_of_str("list_of_tree"),
    //      move || list_of_tree(tree2, Dir2::Left ) )
    drop(tree2);
    sorted2
  }
}

impl Compute<List<Pt2D>,List<Pt2D>> for Quickhull {
  fn compute(inp:List<Pt2D>) -> List<Pt2D> {
    //panic!("TODO")
    inp
  }
}

#[macro_export]
macro_rules! labdef {
  ( $name:expr, $url:expr, $inp:ty, $editst:ty, $out:ty, $dist:ty, $comp:ty ) => {{ 
    Box::new( 
      LabDef
        ::<$inp,$editst,$out,$dist,$comp>
      { 
        identity:$name,
        url:$url,
        input:PhantomData,
        editst:PhantomData,
        output:PhantomData,
        editor:PhantomData,
        archivist:PhantomData
      }) 
  }}
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - 
/// `all_labs`: This is the master list of all labs in the current
/// Adapton Lab catalog (which consists of this module, but which
/// wraps the Adapton crate's collections module).
///
/// To add a new lab, just add a `labdef!` to the `vec!` in
/// this definition.  Doing so generally requires doing at least the
/// following:
/// 
/// 1. Add a new (empty struct) type that implements the `Compute` or
/// `ComputeDemand` trait for some input- and output-type pair.
///
/// 2. The input type must be a type parameter to some implementation
/// of the `Generate` and `Edit` traits, which you must also provide
/// to `labdef!`.  Together, these trait implementations give
/// the distribution of the input (how it is chosen, and how it
/// changes, respectively).
///
/// 3. The other arguments consist of the type of the `Editor` state
/// (e.g., a counter of type `usize` in many cases), and a name and
/// url to display in generated output.  The URL should link to the
/// rustdoc for this module, which in turn provides other related
/// documentation about Adapton Lab and Adapton.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - 
pub fn all_labs() -> Vec<Box<Lab>> {
  return vec![
    labdef!(name_of_str("list-lazy-map"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.LazyMap.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyMap)
      ,
    labdef!(name_of_str("list-lazy-filter"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.LazyFilter.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyFilter)
      ,


    labdef!(name_of_str("list-tree"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.ListTree.html")),
                  List<usize>, usize,
                  Tree<usize>,
                  UniformPrepend<_,_>,
                  ListTree)
      ,
    labdef!(name_of_str("list-tree-max"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.ListTreeMax.html")),
                  List<usize>, usize,
                  usize,
                  UniformPrepend<_,_>,
                  ListTreeMax)
      ,
    labdef!(name_of_str("list-tree-sum"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.ListTreeSum.html")),
                  List<usize>, usize,
                  usize,
                  UniformPrepend<_,_>,
                  ListTreeSum)
      ,

    labdef!(name_of_str("list-eager-mergesort3"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EagerMergesort3.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerMergesort3)
      ,
    labdef!(name_of_str("list-lazy-mergesort3"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.LazyMergesort3.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyMergesort3)
      ,

    labdef!(name_of_str("list-eager-mergesort2"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EagerMergesort2.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerMergesort2)
      ,
    labdef!(name_of_str("list-lazy-mergesort2"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.LazyMergesort2.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyMergesort2)
      ,

    labdef!(name_of_str("list-eager-mergesort1"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EagerMergesort1.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerMergesort1)
      ,
    labdef!(name_of_str("list-lazy-mergesort1"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.LazyMergesort1.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyMergesort1)
      ,

    labdef!(name_of_str("list-eager-map"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EagerMap.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerMap)
      ,
    labdef!(name_of_str("list-eager-filter"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EagerFilter.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerFilter)
      ,
    labdef!(name_of_str("list-reverse"),
                  Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.ListReverse.html")),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  ListReverse)
      ,
    // labdef!(name_of_str("list-quickhull"),
    //               List<Pt2D>, usize,
    //               List<Pt2D>,
    //               UniformPrepend<_,_>,
    //               Quickhull)
    // ,
  ]
}
