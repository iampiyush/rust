// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!
 *
 * # Lattice Variables
 *
 * This file contains generic code for operating on inference variables
 * that are characterized by an upper- and lower-bound.  The logic and
 * reasoning is explained in detail in the large comment in `infer.rs`.
 *
 * The code in here is defined quite generically so that it can be
 * applied both to type variables, which represent types being inferred,
 * and fn variables, which represent function types being inferred.
 * It may eventually be applied to ther types as well, who knows.
 * In some cases, the functions are also generic with respect to the
 * operation on the lattice (GLB vs LUB).
 *
 * Although all the functions are generic, we generally write the
 * comments in a way that is specific to type variables and the LUB
 * operation.  It's just easier that way.
 *
 * In general all of the functions are defined parametrically
 * over a `LatticeValue`, which is a value defined with respect to
 * a lattice.
 */

use core::prelude::*;

use middle::ty::{RegionVid, TyVar};
use middle::ty;
use middle::typeck::isr_alist;
use middle::typeck::infer::*;
use middle::typeck::infer::combine::*;
use middle::typeck::infer::glb::Glb;
use middle::typeck::infer::lub::Lub;
use middle::typeck::infer::unify::*;
use middle::typeck::infer::sub::Sub;
use middle::typeck::infer::lub::Lub;
use middle::typeck::infer::glb::Glb;
use middle::typeck::infer::to_str::InferStr;

use std::list;

trait LatticeValue {
    static fn sub(cf: &CombineFields, a: &self, b: &self) -> ures;
    static fn lub(cf: &CombineFields, a: &self, b: &self) -> cres<self>;
    static fn glb(cf: &CombineFields, a: &self, b: &self) -> cres<self>;
}

type LatticeOp<T> = &fn(cf: &CombineFields, a: &T, b: &T) -> cres<T>;

impl ty::t: LatticeValue {
    static fn sub(cf: &CombineFields, a: &ty::t, b: &ty::t) -> ures {
        Sub(*cf).tys(*a, *b).to_ures()
    }

    static fn lub(cf: &CombineFields, a: &ty::t, b: &ty::t) -> cres<ty::t> {
        Lub(*cf).tys(*a, *b)
    }

    static fn glb(cf: &CombineFields, a: &ty::t, b: &ty::t) -> cres<ty::t> {
        Glb(*cf).tys(*a, *b)
    }
}

impl FnMeta: LatticeValue {
    static fn sub(cf: &CombineFields,
                  a: &FnMeta, b: &FnMeta) -> ures {
        Sub(*cf).fn_metas(a, b).to_ures()
    }

    static fn lub(cf: &CombineFields,
                  a: &FnMeta, b: &FnMeta) -> cres<FnMeta> {
        Lub(*cf).fn_metas(a, b)
    }

    static fn glb(cf: &CombineFields,
                  a: &FnMeta, b: &FnMeta) -> cres<FnMeta> {
        Glb(*cf).fn_metas(a, b)
    }
}

impl CombineFields {
    fn var_sub_var<V:Copy Eq Vid ToStr, T:Copy InferStr LatticeValue>(
        &self,
        vb: &ValsAndBindings<V, Bounds<T>>,
        +a_id: V,
        +b_id: V) -> ures
    {
        /*!
         *
         * Make one variable a subtype of another variable.  This is a
         * subtle and tricky process, as described in detail at the
         * top of infer.rs*/

        // Need to make sub_id a subtype of sup_id.
        let node_a = self.infcx.get(vb, a_id);
        let node_b = self.infcx.get(vb, b_id);
        let a_id = node_a.root;
        let b_id = node_b.root;
        let a_bounds = node_a.possible_types;
        let b_bounds = node_b.possible_types;

        debug!("vars(%s=%s <: %s=%s)",
               a_id.to_str(), a_bounds.inf_str(self.infcx),
               b_id.to_str(), b_bounds.inf_str(self.infcx));

        if a_id == b_id { return uok(); }

        // If both A's UB and B's LB have already been bound to types,
        // see if we can make those types subtypes.
        match (a_bounds.ub, b_bounds.lb) {
            (Some(ref a_ub), Some(ref b_lb)) => {
                let r = self.infcx.try(
                    || LatticeValue::sub(self, a_ub, b_lb));
                match r {
                    Ok(()) => {
                        return Ok(());
                    }
                    Err(_) => { /*fallthrough */ }
                }
            }
            _ => { /*fallthrough*/ }
        }

        // Otherwise, we need to merge A and B so as to guarantee that
        // A remains a subtype of B.  Actually, there are other options,
        // but that's the route we choose to take.

        self.infcx.unify(vb, &node_a, &node_b, |new_root, new_rank| {
            self.set_var_to_merged_bounds(vb, new_root,
                                          &a_bounds, &b_bounds,
                                          new_rank)
        })
    }

    /// make variable a subtype of T
    fn var_sub_t<V:Copy Eq Vid ToStr, T:Copy InferStr LatticeValue>(
        &self,
        vb: &ValsAndBindings<V, Bounds<T>>,
        +a_id: V,
        +b: T) -> ures
    {
        /*!
         *
         * Make a variable (`a_id`) a subtype of the concrete type `b` */

        let node_a = self.infcx.get(vb, a_id);
        let a_id = node_a.root;
        let a_bounds = &node_a.possible_types;
        let b_bounds = &{lb: None, ub: Some(b)};

        debug!("var_sub_t(%s=%s <: %s)",
               a_id.to_str(),
               a_bounds.inf_str(self.infcx),
               b.inf_str(self.infcx));

        self.set_var_to_merged_bounds(
            vb, a_id, a_bounds, b_bounds, node_a.rank)
    }

    fn t_sub_var<V:Copy Eq Vid ToStr, T:Copy InferStr LatticeValue>(
        &self,
        vb: &ValsAndBindings<V, Bounds<T>>,
        +a: T,
        +b_id: V) -> ures
    {
        /*!
         *
         * Make a concrete type (`a`) a subtype of the variable `b_id` */

        let a_bounds = &{lb: Some(a), ub: None};
        let node_b = self.infcx.get(vb, b_id);
        let b_id = node_b.root;
        let b_bounds = &node_b.possible_types;

        debug!("t_sub_var(%s <: %s=%s)",
               a.inf_str(self.infcx),
               b_id.to_str(),
               b_bounds.inf_str(self.infcx));

        self.set_var_to_merged_bounds(
            vb, b_id, a_bounds, b_bounds, node_b.rank)
    }

    fn merge_bnd<T:Copy InferStr LatticeValue>(
        &self,
        a: &Bound<T>,
        b: &Bound<T>,
        lattice_op: LatticeOp<T>)
        -> cres<Bound<T>>
    {
        /*!
         *
         * Combines two bounds into a more general bound. */

        debug!("merge_bnd(%s,%s)",
               a.inf_str(self.infcx),
               b.inf_str(self.infcx));
        let _r = indenter();

        match (*a, *b) {
            (None,          None) => Ok(None),
            (Some(_),       None) => Ok(*a),
            (None,          Some(_)) => Ok(*b),
            (Some(ref v_a), Some(ref v_b)) => {
                do lattice_op(self, v_a, v_b).chain |v| {
                    Ok(Some(v))
                }
            }
        }
    }

    fn set_var_to_merged_bounds<V:Copy Eq Vid ToStr,
                                T:Copy InferStr LatticeValue>(
        &self,
        vb: &ValsAndBindings<V, Bounds<T>>,
        +v_id: V,
        a: &Bounds<T>,
        b: &Bounds<T>,
        rank: uint) -> ures
    {
        /*!
         *
         * Updates the bounds for the variable `v_id` to be the intersection
         * of `a` and `b`.  That is, the new bounds for `v_id` will be
         * a bounds c such that:
         *    c.ub <: a.ub
         *    c.ub <: b.ub
         *    a.lb <: c.lb
         *    b.lb <: c.lb
         * If this cannot be achieved, the result is failure. */

        // Think of the two diamonds, we want to find the
        // intersection.  There are basically four possibilities (you
        // can swap A/B in these pictures):
        //
        //       A         A
        //      / \       / \
        //     / B \     / B \
        //    / / \ \   / / \ \
        //   * *   * * * /   * *
        //    \ \ / /   \   / /
        //     \ B /   / \ / /
        //      \ /   *   \ /
        //       A     \ / A
        //              B

        debug!("merge(%s,%s,%s)",
               v_id.to_str(),
               a.inf_str(self.infcx),
               b.inf_str(self.infcx));
        let _indent = indenter();

        // First, relate the lower/upper bounds of A and B.
        // Note that these relations *must* hold for us to
        // to be able to merge A and B at all, and relating
        // them explicitly gives the type inferencer more
        // information and helps to produce tighter bounds
        // when necessary.
        let () = if_ok!(self.bnds(&a.lb, &b.ub));
        let () = if_ok!(self.bnds(&b.lb, &a.ub));
        let ub = if_ok!(self.merge_bnd(&a.ub, &b.ub, LatticeValue::glb));
        let lb = if_ok!(self.merge_bnd(&a.lb, &b.lb, LatticeValue::lub));
        let bounds = {lb: lb, ub: ub};
        debug!("merge(%s): bounds=%s",
               v_id.to_str(),
               bounds.inf_str(self.infcx));

        // the new bounds must themselves
        // be relatable:
        let () = if_ok!(self.bnds(&bounds.lb, &bounds.ub));
        self.infcx.set(vb, v_id, Root(bounds, rank));
        uok()
    }

    fn bnds<T:Copy InferStr LatticeValue>(
        &self,
        a: &Bound<T>,
        b: &Bound<T>) -> ures
    {
        debug!("bnds(%s <: %s)", a.inf_str(self.infcx),
               b.inf_str(self.infcx));
        let _r = indenter();

        match (*a, *b) {
            (None, None) |
            (Some(_), None) |
            (None, Some(_)) => {
                uok()
            }
            (Some(ref t_a), Some(ref t_b)) => {
                LatticeValue::sub(self, t_a, t_b)
            }
        }
    }
}

// ______________________________________________________________________
// Lattice operations on variables
//
// This is common code used by both LUB and GLB to compute the LUB/GLB
// for pairs of variables or for variables and values.

trait LatticeDir {
    fn combine_fields() -> CombineFields;
    fn bnd<T:Copy>(b: &Bounds<T>) -> Option<T>;
    fn with_bnd<T:Copy>(b: &Bounds<T>, +t: T) -> Bounds<T>;
}

trait TyLatticeDir {
    fn ty_bot(t: ty::t) -> cres<ty::t>;
}

impl Lub: LatticeDir {
    fn combine_fields() -> CombineFields { *self }
    fn bnd<T:Copy>(b: &Bounds<T>) -> Option<T> { b.ub }
    fn with_bnd<T:Copy>(b: &Bounds<T>, +t: T) -> Bounds<T> {
        {ub: Some(t), ..*b}
    }
}

impl Lub: TyLatticeDir {
    fn ty_bot(t: ty::t) -> cres<ty::t> {
        Ok(t)
    }
}

impl Glb: LatticeDir {
    fn combine_fields() -> CombineFields { *self }
    fn bnd<T:Copy>(b: &Bounds<T>) -> Option<T> { b.lb }
    fn with_bnd<T:Copy>(b: &Bounds<T>, +t: T) -> Bounds<T> {
        {lb: Some(t), ..*b}
    }
}

impl Glb: TyLatticeDir {
    fn ty_bot(_t: ty::t) -> cres<ty::t> {
        Ok(ty::mk_bot(self.infcx.tcx))
    }
}

fn super_lattice_tys<L:LatticeDir TyLatticeDir Combine>(
    self: &L,
    a: ty::t,
    b: ty::t) -> cres<ty::t>
{
    debug!("%s.lattice_tys(%s, %s)", self.tag(),
           a.inf_str(self.infcx()),
           b.inf_str(self.infcx()));
    let _r = indenter();

    if a == b {
        return Ok(a);
    }

    let tcx = self.infcx().tcx;

    match (ty::get(a).sty, ty::get(b).sty) {
        (ty::ty_bot, _) => { return self.ty_bot(b); }
        (_, ty::ty_bot) => { return self.ty_bot(a); }

        (ty::ty_infer(TyVar(a_id)), ty::ty_infer(TyVar(b_id))) => {
            let r = if_ok!(lattice_vars(self, &self.infcx().ty_var_bindings,
                                        a_id, b_id,
                                        |x, y| self.tys(*x, *y)));
            return match r {
                VarResult(v) => Ok(ty::mk_var(tcx, v)),
                ValueResult(t) => Ok(t)
            };
        }

        (ty::ty_infer(TyVar(a_id)), _) => {
            return lattice_var_and_t(self, &self.infcx().ty_var_bindings,
                                     a_id, &b,
                                     |x, y| self.tys(*x, *y));
        }

        (_, ty::ty_infer(TyVar(b_id))) => {
            return lattice_var_and_t(self, &self.infcx().ty_var_bindings,
                                     b_id, &a,
                                     |x, y| self.tys(*x, *y));
        }

        _ => {
            return super_tys(self, a, b);
        }
    }
}

type LatticeDirOp<T> = &fn(a: &T, b: &T) -> cres<T>;

enum LatticeVarResult<V,T> {
    VarResult(V),
    ValueResult(T)
}

/**
 * Computes the LUB or GLB of two bounded variables.  These could be any
 * sort of variables, but in the comments on this function I'll assume
 * we are doing an LUB on two type variables.
 *
 * This computation can be done in one of two ways:
 *
 * - If both variables have an upper bound, we may just compute the
 *   LUB of those bounds and return that, in which case we are
 *   returning a type.  This is indicated with a `ValueResult` return.
 *
 * - If the variables do not both have an upper bound, we will unify
 *   the variables and return the unified variable, in which case the
 *   result is a variable.  This is indicated with a `VarResult`
 *   return. */
fn lattice_vars<L:LatticeDir Combine,
                V:Copy Eq Vid ToStr,
                T:Copy InferStr LatticeValue>(
    self: &L,                           // defines whether we want LUB or GLB
    vb: &ValsAndBindings<V, Bounds<T>>, // relevant variable bindings
    +a_vid: V,                          // first variable
    +b_vid: V,                          // second variable
    lattice_dir_op: LatticeDirOp<T>)    // LUB or GLB operation on types
    -> cres<LatticeVarResult<V,T>>
{
    let nde_a = self.infcx().get(vb, a_vid);
    let nde_b = self.infcx().get(vb, b_vid);
    let a_vid = nde_a.root;
    let b_vid = nde_b.root;
    let a_bounds = &nde_a.possible_types;
    let b_bounds = &nde_b.possible_types;

    debug!("%s.lattice_vars(%s=%s <: %s=%s)",
           self.tag(),
           a_vid.to_str(), a_bounds.inf_str(self.infcx()),
           b_vid.to_str(), b_bounds.inf_str(self.infcx()));

    // Same variable: the easy case.
    if a_vid == b_vid {
        return Ok(VarResult(a_vid));
    }

    // If both A and B have an UB type, then we can just compute the
    // LUB of those types:
    let a_bnd = self.bnd(a_bounds), b_bnd = self.bnd(b_bounds);
    match (a_bnd, b_bnd) {
        (Some(ref a_ty), Some(ref b_ty)) => {
            match self.infcx().try(|| lattice_dir_op(a_ty, b_ty) ) {
                Ok(t) => return Ok(ValueResult(t)),
                Err(_) => { /*fallthrough */ }
            }
        }
        _ => {/*fallthrough*/}
    }

    // Otherwise, we need to merge A and B into one variable.  We can
    // then use either variable as an upper bound:
    let cf = self.combine_fields();
    do cf.var_sub_var(vb, a_vid, b_vid).then {
        Ok(VarResult(a_vid))
    }
}

fn lattice_var_and_t<L:LatticeDir Combine,
                     V:Copy Eq Vid ToStr,
                     T:Copy InferStr LatticeValue>(
    self: &L,
    vb: &ValsAndBindings<V, Bounds<T>>,
    +a_id: V,
    b: &T,
    lattice_dir_op: LatticeDirOp<T>)
    -> cres<T>
{
    let nde_a = self.infcx().get(vb, a_id);
    let a_id = nde_a.root;
    let a_bounds = &nde_a.possible_types;

    // The comments in this function are written for LUB, but they
    // apply equally well to GLB if you inverse upper/lower/sub/super/etc.

    debug!("%s.lattice_var_and_t(%s=%s <: %s)",
           self.tag(),
           a_id.to_str(),
           a_bounds.inf_str(self.infcx()),
           b.inf_str(self.infcx()));

    match self.bnd(a_bounds) {
        Some(ref a_bnd) => {
            // If a has an upper bound, return the LUB(a.ub, b)
            debug!("bnd=Some(%s)", a_bnd.inf_str(self.infcx()));
            lattice_dir_op(a_bnd, b)
        }
        None => {
            // If a does not have an upper bound, make b the upper bound of a
            // and then return b.
            debug!("bnd=None");
            let a_bounds = self.with_bnd(a_bounds, *b);
            do self.combine_fields().bnds(&a_bounds.lb, &a_bounds.ub).then {
                self.infcx().set(vb, a_id, Root(a_bounds, nde_a.rank));
                Ok(*b)
            }
        }
    }
}

// ___________________________________________________________________________
// Random utility functions used by LUB/GLB when computing LUB/GLB of
// fn types

fn var_ids<T: Combine>(self: &T, isr: isr_alist) -> ~[RegionVid] {
    let mut result = ~[];
    for list::each(isr) |pair| {
        match pair.second() {
            ty::re_infer(ty::ReVar(r)) => { result.push(r); }
            r => {
                self.infcx().tcx.sess.span_bug(
                    self.span(),
                    fmt!("Found non-region-vid: %?", r));
            }
        }
    }
    return result;
}

fn is_var_in_set(new_vars: &[RegionVid], r: ty::Region) -> bool {
    match r {
        ty::re_infer(ty::ReVar(ref v)) => new_vars.contains(v),
        _ => false
    }
}
