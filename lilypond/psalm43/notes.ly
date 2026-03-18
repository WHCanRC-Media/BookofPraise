melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'2 a2 a2 g4 g4 c4 c4 b2 a2 \break

  % Line 2
  r4 g4 a2 c2 b4 a2 g2 fis4 g2 r2 \break

  % Line 3
  b2 b4 b4 a4 a4 b4 d2 cis4 d2 r2 \break

  % Line 4
  a2 a4 a4 b2 a2 g4 fis4 e2 d2 r2 \break

  % Line 5
  b'2 d4 c4 b4 a4 b2 c2 b4 \break

  % Line 6
  b4 a4 g4 g4 fis4 g1 \bar "|."
}
